use std::{
    env,
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
};

use burn::{
    record::{FullPrecisionSettings, PrecisionSettings},
    tensor::{DataSerialize, Element},
};

use crate::{
    burn::{
        graph::BurnGraph,
        node::{
            batch_norm::BatchNormNode,
            binary::BinaryNode,
            concat::ConcatNode,
            constant::{ConstantNode, ConstantValue, TensorValue},
            conv2d::Conv2dNode,
            dropout::DropoutNode,
            global_avg_pool::GlobalAvgPoolNode,
            linear::LinearNode,
            matmul::MatmulNode,
            max_pool2d::MaxPool2dNode,
            reshape::ReshapeNode,
            unary::UnaryNode,
        },
        ScalarKind, ScalarType, TensorKind, TensorType, Type,
    },
    format_tokens,
    logger::init_log,
    onnx::{
        ir::{AttributeValue, Node, NodeType},
        op_configuration::{
            batch_norm_config, conv2d_config, flatten_config, linear_config, log_softmax_config,
            max_pool2d_config,
        },
    },
};

use super::{
    from_onnx::parse_onnx,
    ir::{ArgType, Argument, ElementType, ONNXGraph, State, StateType, Tensor, TensorData},
    op_configuration::{concat_config, dropout_config},
};

/// Generate code and states from `.onnx` files and save them to the `out_dir`.
#[derive(Debug, Default)]
pub struct ModelGen {
    out_dir: Option<PathBuf>,
    /// List of onnx files to generate source code from.
    inputs: Vec<PathBuf>,
    development: bool,
}

impl ModelGen {
    /// Create a new `ModelGen`.
    pub fn new() -> Self {
        init_log().ok(); // Error when init multiple times are ignored.
        Self::default()
    }

    /// Set output directory.
    pub fn out_dir(&mut self, out_dir: &str) -> &mut Self {
        self.out_dir = Some(Path::new(out_dir).into());
        self
    }

    /// Add input file.
    pub fn input(&mut self, input: &str) -> &mut Self {
        self.inputs.push(input.into());
        self
    }

    /// Set development mode.
    ///
    /// If this is set to true, the generated model will be saved as `.graph.txt` files and model
    /// states will be saved as `.json` file.
    pub fn development(&mut self, development: bool) -> &mut Self {
        self.development = development;
        self
    }

    /// Run code generation.
    ///
    /// This function is intended to be called from `build.rs` script.
    pub fn run_from_script(&self) {
        self.run(true);
    }

    /// Run code generation.
    ///
    /// This function is intended to be called from CLI.
    pub fn run_from_cli(&self) {
        self.run(false);
    }

    /// Run code generation.
    fn run(&self, is_build_script: bool) {
        log::info!("Starting to convert ONNX to Burn");

        // prepend the out_dir to the cargo_out_dir if this is a build script
        let out_dir = if is_build_script {
            let cargo_out_dir = env::var("OUT_DIR").expect("OUT_DIR env is not set");
            let mut path = PathBuf::from(cargo_out_dir);

            // // Append the out_dir to the cargo_out_dir
            path.push(self.out_dir.clone().unwrap());
            path
        } else {
            self.out_dir.as_ref().expect("out_dir is not set").clone()
        };

        log::debug!("Output directory: {:?}", out_dir);

        create_dir_all(&out_dir).unwrap();

        for input in self.inputs.iter() {
            let file_name = input.file_stem().unwrap();
            let out_file: PathBuf = out_dir.join(file_name);

            log::info!("Converting {:?}", input);
            log::debug!("Input file name: {:?}", file_name);
            log::debug!("Output file: {:?}", out_file);

            Self::generate_model(self.development, input, out_file);
        }

        log::info!("Finished converting ONNX to Burn");
    }

    /// Generate model source code and model state.
    fn generate_model(development: bool, input: &PathBuf, out_file: PathBuf) {
        log::info!("Generating model from {:?}", input);
        log::debug!("Development mode: {:?}", development);
        log::debug!("Output file: {:?}", out_file);

        let graph = parse_onnx(input.as_ref());

        if development {
            // export the graph
            let debug_graph = format!("{:#?}", graph);
            let graph_file = out_file.with_extension("graph.txt");
            log::debug!("Writing debug graph file: {:?}", graph_file);
            fs::write(graph_file, debug_graph).unwrap();
        }

        let graph = graph
            .into_burn::<FullPrecisionSettings>()
            .with_record(
                out_file.clone(),
                development,
                "burn::record::FullPrecisionSettings",
            )
            .with_new_fn(true)
            .with_blank_space(true)
            .with_top_comment(Some(format!(
                "Generated from ONNX {input:?} by burn-import"
            )));

        let code_str = format_tokens(graph.codegen());
        fs::write(out_file.with_extension("rs"), code_str).unwrap();

        log::info!("Model generated");
    }
}

impl ONNXGraph {
    /// Converts ONNX graph to Burn graph.
    pub fn into_burn<PS: PrecisionSettings + 'static>(self) -> BurnGraph<PS> {
        let mut graph = BurnGraph::<PS>::default();

        for node in self.nodes {
            match node.node_type {
                NodeType::Add => graph.register(Self::add_conversion(node)),
                NodeType::Sub => graph.register(Self::sub_conversion(node)),
                NodeType::Mul => graph.register(Self::mul_conversion(node)),
                NodeType::Div => graph.register(Self::div_conversion(node)),
                NodeType::Equal => graph.register(Self::equal_conversion(node)),
                NodeType::Conv2d => graph.register(Self::conv2d_conversion::<PS>(node)),
                NodeType::MaxPool2d => graph.register(Self::max_pool2d_conversion(node)),
                NodeType::MatMul => graph.register(Self::matmul_conversion(node)),
                NodeType::Linear => graph.register(Self::linear_conversion::<PS>(node)),
                NodeType::BatchNormalization => {
                    graph.register(Self::batch_norm_conversion::<PS>(node))
                }
                NodeType::Relu => graph.register(Self::relu_conversion(node)),
                NodeType::Flatten => graph.register(Self::flatten_conversion(node)),
                NodeType::LogSoftmax => graph.register(Self::log_softmax_conversion(node)),
                NodeType::Constant => graph.register(Self::constant_conversion::<PS>(node)),
                NodeType::Reshape => graph.register(Self::reshape_conversion(node)),
                NodeType::Sigmoid => graph.register(Self::sigmoid_conversion(node)),
                NodeType::Transpose => graph.register(Self::transpose_conversion(node)),
                NodeType::Concat => graph.register(Self::concat_conversion(node)),
                NodeType::Cast => graph.register(Self::cast_conversion(node)),
                NodeType::Dropout => graph.register(Self::dropout_conversion(node)),
                NodeType::GlobalAveragePool => {
                    graph.register(Self::global_avg_pool_conversion(node))
                }
                _ => panic!("Unsupported node conversion {}", node.node_type),
            }
        }

        // Get input and output names
        let input_names = self
            .inputs
            .iter()
            .map(|input| input.name.clone())
            .collect::<Vec<_>>();
        let output_names = self
            .outputs
            .iter()
            .map(|output| output.name.clone())
            .collect::<Vec<_>>();

        // Register inputs and outputs with the graph
        graph.register_input_output(input_names, output_names);

        graph
    }

    fn constant_conversion<PS: PrecisionSettings>(mut node: Node) -> ConstantNode<PS> {
        let output = node.outputs.get(0).unwrap();

        let value = node.attrs.remove("value").unwrap();

        let value = match value {
            AttributeValue::Float32(val) => ConstantValue::Float32(val),
            AttributeValue::Int64(val) => ConstantValue::Int64(val),
            AttributeValue::Tensor(tensor) => {
                if tensor.dim == 0 {
                    // Treat zero dim tensor as scalar value by extracting the first element
                    // because PyTorch/ONNX uses zero dim tensor for scalar values
                    match tensor.data.unwrap() {
                        TensorData::Float32(val) => ConstantValue::Float32(val[0]),
                        TensorData::Float64(val) => ConstantValue::Float64(val[0]),
                        TensorData::Int32(val) => ConstantValue::Int32(val[0]),
                        TensorData::Int64(val) => ConstantValue::Int64(val[0]),
                        TensorData::Bool(val) => ConstantValue::Bool(val[0]),
                        _ => panic!(
                            "Unsupported zero dim constant tensor type: {:?} ",
                            tensor.elem_type
                        ),
                    }
                } else {
                    let ds = match tensor.elem_type {
                        ElementType::Float32 | ElementType::Float64 => TensorValue::Float(
                            tensor.clone().into_data_serialize::<PS::FloatElem>(),
                        ),
                        ElementType::Int32 | ElementType::Int64 => {
                            TensorValue::Int(tensor.clone().into_data_serialize::<PS::IntElem>())
                        }
                        _ => panic!("Unsupported constant tensor type: {:?} ", tensor.elem_type),
                    };

                    ConstantValue::<PS>::Tensor(
                        TensorType::new(
                            node.name.clone(),
                            tensor.dim,
                            tensor.elem_type.into(),
                            tensor.shape,
                        ),
                        ds,
                    )
                }
            }
            _ => panic!("Unsupported constant value: {:?} ", value),
        };

        ConstantNode::new(node.name.clone(), value, output.to_type())
    }

    fn add_conversion(node: Node) -> BinaryNode {
        let lhs = node.inputs.get(0).unwrap().to_type();
        let rhs = node.inputs.get(1).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();

        BinaryNode::add(lhs, rhs, output)
    }

    fn sub_conversion(node: Node) -> BinaryNode {
        let lhs = node.inputs.get(0).unwrap().to_type();
        let rhs = node.inputs.get(1).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();

        BinaryNode::sub(lhs, rhs, output)
    }

    fn mul_conversion(node: Node) -> BinaryNode {
        let lhs = node.inputs.get(0).unwrap().to_type();
        let rhs = node.inputs.get(1).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();

        BinaryNode::mul(lhs, rhs, output)
    }

    fn div_conversion(node: Node) -> BinaryNode {
        let lhs = node.inputs.get(0).unwrap().to_type();
        let rhs = node.inputs.get(1).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();

        BinaryNode::div(lhs, rhs, output)
    }

    fn matmul_conversion(node: Node) -> MatmulNode {
        let lhs = node.inputs.get(0).unwrap().to_tensor_type();
        let rhs = node.inputs.get(1).unwrap().to_tensor_type();
        let output = node.outputs.get(0).unwrap().to_tensor_type();

        MatmulNode::new(lhs, rhs, output)
    }

    fn equal_conversion(node: Node) -> BinaryNode {
        let lhs = node.inputs.get(0).unwrap().to_type();
        let rhs = node.inputs.get(1).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();

        BinaryNode::equal(lhs, rhs, output)
    }

    fn relu_conversion(node: Node) -> UnaryNode {
        let input = node.inputs.get(0).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();

        UnaryNode::relu(input, output)
    }

    fn flatten_conversion(node: Node) -> UnaryNode {
        let input = node.inputs.get(0).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();
        let (start_dim, end_dim) = flatten_config(&node);

        UnaryNode::flatten(input, output, start_dim, end_dim)
    }

    fn transpose_conversion(node: Node) -> UnaryNode {
        let input = node.inputs.get(0).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();

        UnaryNode::transpose(input, output)
    }

    fn cast_conversion(node: Node) -> UnaryNode {
        let input = node.inputs.get(0).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();

        UnaryNode::cast(input, output)
    }

    fn reshape_conversion(mut node: Node) -> ReshapeNode {
        let input = node.inputs.get(0).unwrap().to_tensor_type();
        let output = node.outputs.get(0).unwrap().to_tensor_type();
        let shape = extract_next_data_serialize::<i64>(&mut node).unwrap();

        ReshapeNode::new(
            input,
            output,
            shape.value.iter().map(|item| *item as usize).collect(),
        )
    }

    fn sigmoid_conversion(node: Node) -> UnaryNode {
        let input = node.inputs.get(0).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();

        UnaryNode::sigmoid(input, output)
    }

    fn log_softmax_conversion(node: Node) -> UnaryNode {
        let input = node.inputs.get(0).unwrap().to_type();
        let output = node.outputs.get(0).unwrap().to_type();
        let dim = log_softmax_config(&node);

        UnaryNode::log_softmax(input, output, dim)
    }

    fn concat_conversion(node: Node) -> ConcatNode {
        let inputs = node
            .inputs
            .iter()
            .map(|input| input.to_tensor_type())
            .collect();

        let output = node.outputs.get(0).unwrap().to_tensor_type();
        let dim = concat_config(&node);

        ConcatNode::new(inputs, output, dim)
    }

    fn linear_conversion<PS: PrecisionSettings>(mut node: Node) -> LinearNode<PS> {
        let name = &node.name;
        let input = node.inputs.get(0).unwrap().to_tensor_type();
        let output = node.outputs.get(0).unwrap().to_tensor_type();
        let config = linear_config(&node);

        let bias = node.states.len() == 2;
        let weight = node.states.remove(0).into_data_serialize::<PS::FloatElem>();

        let bias = match bias {
            true => Some(node.states.remove(0).into_data_serialize::<PS::FloatElem>()),
            false => None,
        };

        LinearNode::new(name, input, output, weight, bias, config)
    }

    fn dropout_conversion(node: Node) -> DropoutNode {
        let name = &node.name;
        let input = node.inputs.get(0).unwrap().to_tensor_type();
        let output = node.outputs.get(0).unwrap().to_tensor_type();
        let config = dropout_config(&node);

        DropoutNode::new(name, input, output, config)
    }

    fn batch_norm_conversion<PS: PrecisionSettings>(mut node: Node) -> BatchNormNode<PS> {
        let config = batch_norm_config(&node);
        let input = node.inputs.get(0).unwrap().to_tensor_type();
        let output = node.outputs.get(0).unwrap().to_tensor_type();
        let dim = input.dim - 2;

        let gamma =
            extract_next_data_serialize::<PS::FloatElem>(&mut node).expect("Gamma is required");
        let beta =
            extract_next_data_serialize::<PS::FloatElem>(&mut node).expect("Gamma is required");
        let running_mean = extract_next_data_serialize::<PS::FloatElem>(&mut node)
            .expect("Running mean is required");
        let running_var = extract_next_data_serialize::<PS::FloatElem>(&mut node)
            .expect("Running var is required");

        let name = &node.name;

        BatchNormNode::new(
            dim,
            name,
            input,
            output,
            gamma,
            beta,
            running_mean,
            running_var,
            config,
        )
    }

    fn conv2d_conversion<PS: PrecisionSettings>(mut node: Node) -> Conv2dNode<PS> {
        let input = node.inputs.get(0).unwrap().to_tensor_type();
        let output = node.outputs.get(0).unwrap().to_tensor_type();
        let config = conv2d_config(&node);

        let bias = node.states.len() == 2;
        let weight = extract_next_data_serialize::<PS::FloatElem>(&mut node).unwrap();
        let bias = match bias {
            true => Some(extract_next_data_serialize::<PS::FloatElem>(&mut node)).unwrap(),
            false => None,
        };

        let name = &node.name;
        Conv2dNode::<PS>::new(name, input, output, weight, bias, config)
    }

    fn max_pool2d_conversion(node: Node) -> MaxPool2dNode {
        let input = node.inputs.get(0).unwrap().to_tensor_type();
        let output = node.outputs.get(0).unwrap().to_tensor_type();
        let config = max_pool2d_config(&node);

        let name = &node.name;
        MaxPool2dNode::new(name, input, output, config)
    }

    fn global_avg_pool_conversion(node: Node) -> GlobalAvgPoolNode {
        let input = node.inputs.get(0).unwrap().to_tensor_type();
        let output = node.outputs.get(0).unwrap().to_tensor_type();

        let name = &node.name;

        GlobalAvgPoolNode::new(name, input, output)
    }
}

fn extract_next_data_serialize<E: Element>(node: &mut Node) -> Option<DataSerialize<E>> {
    if node.states.is_empty() {
        return None;
    }

    Some(node.states.remove(0).into_data_serialize::<E>())
}

impl State {
    pub fn into_data_serialize<E: Element>(self) -> DataSerialize<E> {
        match self.ty {
            StateType::Tensor(tensor) => tensor.into_data_serialize(),
        }
    }
}

impl Argument {
    pub fn to_tensor_type(&self) -> TensorType {
        match &self.ty {
            ArgType::Tensor(tensor) => TensorType::new_float(self.name.clone(), tensor.dim),
            _ => panic!("Can't transform to tensor."),
        }
    }

    pub fn to_type(&self) -> Type {
        match &self.ty {
            ArgType::Tensor(tensor) => {
                // Treat tensor with dim 0 as scalar
                if tensor.dim == 0 {
                    // FIXME Convert to correct scalar type (@antimora 8/1/2023)
                    // Currently it's not dangerous because we don't use specific scalar type
                    Type::Scalar(ScalarType::new(self.name.clone(), ScalarKind::Float64))
                } else {
                    Type::Tensor(TensorType::new_float(self.name.clone(), tensor.dim))
                }
            }

            ArgType::Scalar(elem_type) => {
                Type::Scalar(ScalarType::new(self.name.clone(), elem_type.into()))
            }
            ArgType::Shape(_shape) => panic!("Can't transform shape to tensor."),
        }
    }
}

impl From<&ElementType> for ScalarKind {
    fn from(elem_type: &ElementType) -> Self {
        match elem_type {
            ElementType::Float32 => ScalarKind::Float32,
            ElementType::Float64 => ScalarKind::Float64,
            ElementType::Int32 => ScalarKind::Int32,
            ElementType::Int64 => ScalarKind::Int64,
            ElementType::Bool => ScalarKind::Bool,
            ElementType::String => panic!("String tensor unsupported"),
            ElementType::Float16 => panic!("Float16 tensor unsupported"),
        }
    }
}

impl From<ElementType> for TensorKind {
    fn from(elem_type: ElementType) -> Self {
        match elem_type {
            ElementType::Float32 => TensorKind::Float,
            ElementType::Float64 => TensorKind::Float,
            ElementType::Int32 => TensorKind::Int,
            ElementType::Int64 => TensorKind::Int,
            ElementType::Bool => TensorKind::Bool,
            _ => panic!("Unsupported tensor type"),
        }
    }
}

impl Tensor {
    pub fn into_data_serialize<E: Element>(self) -> DataSerialize<E> {
        let data = self.data.expect("Data to be provided.");

        match data {
            TensorData::Float16(val) => DataSerialize::new(val, self.shape.unwrap()).convert(),
            TensorData::Float32(val) => DataSerialize::new(val, self.shape.unwrap()).convert(),
            TensorData::Float64(val) => DataSerialize::new(val, self.shape.unwrap()).convert(),
            TensorData::Int32(val) => DataSerialize::new(val, self.shape.unwrap()).convert(),
            TensorData::Int64(val) => DataSerialize::new(val, self.shape.unwrap()).convert(),
            TensorData::String(_) => panic!("String tensor unsupported"),
            TensorData::Bool(_) => panic!("Bool tensor unsupported"),
        }
    }
}
