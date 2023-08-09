use crate::{backend::Backend, Shape};

use super::{MaxPool1dBackward, MaxPool1dWithIndices};

pub(crate) fn avg_pool1d_from_2d<B: Backend>(
    x: B::TensorPrimitive<3>,
    kernel_size: usize,
    stride: usize,
    padding: usize,
) -> B::TensorPrimitive<3> {
    let [batch_size, channels, length] = B::shape(&x).dims;

    let x = B::reshape(x, Shape::from([batch_size, channels, length, 1]));
    let x = B::avg_pool2d(x, [kernel_size, 1], [stride, 1], [padding, 0]);

    let [batch_size, channels, length, _] = B::shape(&x).dims;

    B::reshape(x, Shape::from([batch_size, channels, length]))
}

pub(crate) fn avg_pool1d_backward_from_2d<B: Backend>(
    x: B::TensorPrimitive<3>,
    grad: B::TensorPrimitive<3>,
    kernel_size: usize,
    stride: usize,
    padding: usize,
) -> B::TensorPrimitive<3> {
    let [batch_size, channels, length_in] = B::shape(&x).dims;
    let [_, _, length_out] = B::shape(&grad).dims;

    let x = B::reshape(x, Shape::from([batch_size, channels, length_in, 1]));
    let grad_x = B::reshape(grad, Shape::from([batch_size, channels, length_out, 1]));

    let grad_x = B::avg_pool2d_backward(x, grad_x, [kernel_size, 1], [stride, 1], [padding, 0]);

    B::reshape(grad_x, Shape::from([batch_size, channels, length_in]))
}

pub(crate) fn adaptive_avg_pool1d_from_2d<B: Backend>(
    x: B::TensorPrimitive<3>,
    output_size: usize,
) -> B::TensorPrimitive<3> {
    let [batch_size, channels, length] = B::shape(&x).dims;

    let x = B::reshape(x, Shape::from([batch_size, channels, length, 1]));
    let x = B::adaptive_avg_pool2d(x, [output_size, 1]);

    let [batch_size, channels, length, _] = B::shape(&x).dims;

    B::reshape(x, Shape::from([batch_size, channels, length]))
}

pub(crate) fn adaptive_avg_pool1d_backward_from_2d<B: Backend>(
    x: B::TensorPrimitive<3>,
    grad: B::TensorPrimitive<3>,
) -> B::TensorPrimitive<3> {
    let [batch_size, channels, length_in] = B::shape(&x).dims;
    let [_, _, length_out] = B::shape(&grad).dims;

    let x = B::reshape(x, Shape::from([batch_size, channels, length_in, 1]));
    let grad_x = B::reshape(grad, Shape::from([batch_size, channels, length_out, 1]));

    let grad_x = B::adaptive_avg_pool2d_backward(x, grad_x);

    B::reshape(grad_x, Shape::from([batch_size, channels, length_in]))
}

pub(crate) fn max_pool1d_from_2d<B: Backend>(
    x: B::TensorPrimitive<3>,
    kernel_size: usize,
    stride: usize,
    padding: usize,
) -> B::TensorPrimitive<3> {
    let [batch_size, channels, length] = B::shape(&x).dims;

    let x = B::reshape(x, Shape::from([batch_size, channels, length, 1]));
    let x = B::max_pool2d(x, [kernel_size, 1], [stride, 1], [padding, 0]);

    let [batch_size, channels, length, _] = B::shape(&x).dims;

    B::reshape(x, Shape::from([batch_size, channels, length]))
}

pub(crate) fn max_pool1d_with_indices_from_2d<B: Backend>(
    x: B::TensorPrimitive<3>,
    kernel_size: usize,
    stride: usize,
    padding: usize,
) -> MaxPool1dWithIndices<B> {
    let [batch_size, channels, length] = B::shape(&x).dims;

    let x = B::reshape(x, Shape::from([batch_size, channels, 1, length]));
    let x = B::max_pool2d_with_indices(x, [1, kernel_size], [1, stride], [0, padding]);
    let [batch_size, channels, _, length] = B::shape(&x.output).dims;
    let output = B::reshape(x.output, Shape::from([batch_size, channels, length]));
    let indices = B::int_reshape(
        x.indices.clone(),
        Shape::from([batch_size, channels, length]),
    );
    MaxPool1dWithIndices::new(output, indices)
}

pub(crate) fn max_pool1d_with_indices_backward_from_2d<B: Backend>(
    x: B::TensorPrimitive<3>,
    kernel_size: usize,
    stride: usize,
    padding: usize,
    output_grad: B::TensorPrimitive<3>,
    indices: B::IntTensorPrimitive<3>,
) -> MaxPool1dBackward<B> {
    let [batch_size, channels, length_in] = B::shape(&x).dims;
    let [_, _, length_out] = B::shape(&output_grad).dims;

    let x = B::reshape(x, Shape::from([batch_size, channels, length_in, 1]));
    let grad_x = B::reshape(
        output_grad,
        Shape::from([batch_size, channels, length_out, 1]),
    );
    let indices = B::int_reshape(indices, Shape::from([batch_size, channels, length_out, 1]));

    let grad_x = B::max_pool2d_with_indices_backward(
        x,
        [kernel_size, 1],
        [stride, 1],
        [padding, 0],
        grad_x,
        indices,
    )
    .x_grad;

    MaxPool1dBackward::new(B::reshape(
        grad_x,
        Shape::from([batch_size, channels, length_in]),
    ))
}
