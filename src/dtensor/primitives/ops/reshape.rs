use crate::dtensor::{self, primitives::ops::builders, primitives::Tensor};
use wgpu;

const ENTRY_POINT: &str = "reshape";
const WORKGROUP_SIZE: usize = 64;

// Functional implementation
pub async fn reshape<'op>(input: &Tensor<'op>, shape: &[usize]) -> Tensor<'op> {
    let wgpu_device = input.device();
    let (device, _) = wgpu_device;

    let result = Tensor::of_shape(shape, wgpu_device).await;

    let pipeline_descriptor = builders::TensorOpDescriptor {
        inputs: &[builders::TensorDescriptor {
            name: "input",
            tensor: input,
        }],
        output: builders::TensorDescriptor {
            name: "result",
            tensor: &result,
        },
        entry_point: ENTRY_POINT,
    };

    let shader_source = generate_wgsl_shader(&pipeline_descriptor);
    let compiled_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    builders::build_op_pipeline(&pipeline_descriptor, &compiled_shader, wgpu_device);
    result
}

fn generate_wgsl_shader(pipeline_descriptor: &builders::TensorOpDescriptor) -> String {
    format!(
        "
{shader_interface}

@compute @workgroup_size({workgroup_size}, 1, 1)
fn {entry_point}(@builtin(global_invocation_id) global_id: vec3u) {{
  // Guard against out-of-bounds work group sizes
  if (global_id.x >= result_metadata.length) {{
    return;
  }}

  {workarounds}

  // Unmapped index
  let index: u32 = global_id.x;

  // Map index from the contiguous result offset to the strided input offset
  var contiguous_offset: u32 = index;
  var mapped_offset: u32 = 0u;
  for (var i = 0u; i < input_metadata.rank; i++) {{
    mapped_offset = mapped_offset + (contiguous_offset / input_contiguous_stride[i] * input_stride[i]);
    contiguous_offset = contiguous_offset % input_contiguous_stride[i];
  }}

  result[index] = input[mapped_offset];
}}
",
        shader_interface = builders::define_shader_interface(pipeline_descriptor.inputs, &pipeline_descriptor.output),
        workgroup_size = WORKGROUP_SIZE,
        entry_point = pipeline_descriptor.entry_point,
        workarounds = builders::shader_workaround_1976(pipeline_descriptor),
    )
}