use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use num_traits::AsPrimitive;
use uuid::Uuid;

use crate::FILE_MANAGER;

use super::*;

static ID_GENERATOR: AtomicU32 = AtomicU32::new(0);

pub type TensorType = f32;

#[derive(Clone, Debug)]
pub struct Tensor(Rc<TensorInternals>);

#[derive(Clone, Debug)]
pub struct TensorInternals {
    pub id: u32,
    view: TensorView,
    pub data: TensorInput,
}

impl TensorInternals {
    pub fn new(view: TensorView, data: TensorInput) -> TensorInternals {
        TensorInternals {
            id: ID_GENERATOR.fetch_add(1, Ordering::Relaxed),
            view: view,
            data: data,
        }
    }
}

impl Tensor {
    pub fn scalar<T: AsPrimitive<TensorType>>(data: T) -> Tensor {
        let data = vec![data.as_()];
        let shape = vec![1];
        Tensor::from_contiguous(&data, &shape)
    }

    pub fn from_contiguous(data: &[TensorType], shape: &[ViewType]) -> Tensor {
        let identifier = Uuid::new_v4().to_string();
        let path = Path::new(&identifier);
        FILE_MANAGER
            .lock()
            .unwrap()
            .create_with_bytes(path, bytemuck::cast_slice(data))
            .unwrap();

        Tensor(Rc::new(TensorInternals::new(
            TensorView::from_shape(shape),
            TensorInput::from_raw(path.to_path_buf(), 0),
        )))
    }
}