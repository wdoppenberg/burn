#[burn_tensor_testgen::testgen(module_max_pool1d)]
mod tests {
    use super::*;
    use burn_tensor::module::{max_pool1d, max_pool1d_with_indices};
    use burn_tensor::{backend::Backend, Data, Tensor};

    type IntElem = <TestBackend as Backend>::IntElem;

    #[test]
    fn test_max_pool1d_simple() {
        let batch_size = 2;
        let channels_in = 2;
        let kernel_size = 3;
        let padding = 1;
        let stride = 1;

        let x = TestTensor::from_floats([[
            [0.9861, 0.5474, 0.4477, 0.0732, 0.3548, 0.8221],
            [0.8148, 0.5474, 0.9490, 0.7890, 0.5537, 0.5689],
        ]]);
        let y = TestTensor::from_floats([[
            [0.9861, 0.9861, 0.5474, 0.4477, 0.8221, 0.8221],
            [0.8148, 0.9490, 0.9490, 0.9490, 0.7890, 0.5689],
        ]]);

        let output = max_pool1d(x, kernel_size, stride, padding);

        y.to_data().assert_approx_eq(&output.into_data(), 3);
    }

    #[test]
    fn test_max_pool1d_different_padding_stride_kernel() {
        let batch_size = 1;
        let channels_in = 1;
        let kernel_size = 3;
        let padding = 1;
        let stride = 2;

        let x = TestTensor::from_floats([[[0.6309, 0.6112, 0.6998, 0.4708]]]);
        let y = TestTensor::from_floats([[[0.6309, 0.6998]]]);

        let output = max_pool1d(x, kernel_size, stride, padding);

        y.to_data().assert_approx_eq(&output.into_data(), 3);
    }

    #[test]
    fn test_max_pool1d_with_neg() {
        let batch_size = 1;
        let channels_in = 1;
        let kernel_size = 3;
        let padding = 1;
        let stride = 1;

        let x = TestTensor::from_floats([[[-0.6309, -0.6112, -0.6998, -0.4708]]]);
        let y = TestTensor::from_floats([[[-0.6112, -0.6112, -0.4708, -0.4708]]]);

        let output = max_pool1d(x, kernel_size, stride, padding);

        y.to_data().assert_approx_eq(&output.into_data(), 3);
    }

    #[test]
    fn test_max_pool1d_with_indices() {
        let batch_size = 1;
        let channels_in = 1;
        let kernel_size = 2;
        let padding = 1;
        let stride = 1;

        let x = TestTensor::from_floats([[[0.2479, 0.6386, 0.3166, 0.5742]]]);
        let indices = Data::<IntElem, 3>::from([[[0, 1, 1, 3, 3]]]);
        let y = TestTensor::from_floats([[[0.2479, 0.6386, 0.6386, 0.5742, 0.5742]]]);

        let (output, output_indices) = max_pool1d_with_indices(x, kernel_size, stride, padding);

        y.to_data().assert_approx_eq(&output.into_data(), 3);
        assert_eq!(indices.value, output_indices.into_data().value);
    }

    #[test]
    fn test_max_pool1d_complex() {
        let batch_size = 1;
        let channels_in = 1;
        let kernel_size = 4;
        let padding = 2;
        let stride = 1;

        let x = TestTensor::from_floats([[[0.5388, 0.0676, 0.7122, 0.8316, 0.0653]]]);
        let indices = Data::<IntElem, 3>::from([[[0, 2, 3, 3, 3, 3]]]);
        let y = TestTensor::from_floats([[[0.5388, 0.7122, 0.8316, 0.8316, 0.8316, 0.8316]]]);

        let (output, output_indices) = max_pool1d_with_indices(x, kernel_size, stride, padding);

        y.to_data().assert_approx_eq(&output.into_data(), 3);
        assert_eq!(indices.value, output_indices.into_data().value);
    }
}
