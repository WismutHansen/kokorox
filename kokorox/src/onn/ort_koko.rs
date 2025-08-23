use std::borrow::Cow;
use std::cell::RefCell;

use ndarray::{ArrayBase, IxDyn, OwnedRepr};
use ort::{
    session::{Session, SessionInputValue, SessionInputs, SessionOutputs},
    value::{Tensor, Value},
};

use super::ort_base;
use ort_base::OrtBase;

pub struct OrtKoko {
    sess: Option<RefCell<Session>>,
}

unsafe impl Send for OrtKoko {}
unsafe impl Sync for OrtKoko {}
impl ort_base::OrtBase for OrtKoko {
    fn set_sess(&mut self, sess: Session) {
        self.sess = Some(RefCell::new(sess));
    }

    fn sess(&self) -> Option<&RefCell<Session>> {
        self.sess.as_ref()
    }
}
impl OrtKoko {
    pub fn new(model_path: String) -> Result<Self, String> {
        let mut instance = OrtKoko { sess: None };
        instance.load_model(model_path)?;
        Ok(instance)
    }

    pub fn infer(
        &self,
        tokens: Vec<Vec<i64>>,
        styles: Vec<Vec<f32>>,
        speed: f32,
    ) -> Result<ArrayBase<OwnedRepr<f32>, IxDyn>, Box<dyn std::error::Error>> {
        // inference koko
        // token, styles, speed
        // 1,N 1,256
        // [[0, 56, 51, 142, 156, 69, 63, 3, 16, 61, 4, 16, 156, 51, 4, 16, 62, 77, 156, 51, 86, 5, 0]]

        // Add proper padding as per original implementation: [0, *tokens, 0]
        let mut tokens = tokens;
        if !tokens.is_empty() && !tokens[0].is_empty() {
            let mut padded_tokens = vec![0]; // Start with padding token
            padded_tokens.extend(tokens[0].clone()); // Add original tokens
            padded_tokens.push(0); // End with padding token
            tokens[0] = padded_tokens;
        }

        let shape = [tokens.len(), tokens[0].len()];
        let tokens_flat: Vec<i64> = tokens.into_iter().flatten().collect();
        let tokens = Tensor::from_array((shape, tokens_flat))?;
        let tokens_value: SessionInputValue = SessionInputValue::Owned(Value::from(tokens));

        let shape_style = [styles.len(), styles[0].len()];
        eprintln!("shape_style: {shape_style:?}");
        let style_flat: Vec<f32> = styles.into_iter().flatten().collect();
        let style = Tensor::from_array((shape_style, style_flat))?;
        let style_value: SessionInputValue = SessionInputValue::Owned(Value::from(style));

        let speed = vec![speed; 1];
        let speed = Tensor::from_array(([1], speed))?;
        let speed_value: SessionInputValue = SessionInputValue::Owned(Value::from(speed));

        let inputs: Vec<(Cow<str>, SessionInputValue)> = vec![
            (Cow::Borrowed("tokens"), tokens_value),
            (Cow::Borrowed("style"), style_value),
            (Cow::Borrowed("speed"), speed_value),
        ];

        if let Some(sess_cell) = &self.sess {
            let mut sess = sess_cell.borrow_mut();
            let outputs: SessionOutputs = sess.run(SessionInputs::from(inputs))?;
            let (tensor_shape, data) = outputs["audio"]
                .try_extract_tensor::<f32>()
                .expect("Failed to extract tensor");
            let dims: Vec<usize> = tensor_shape.iter().map(|&dim| dim as usize).collect();
            
            // Debug: Check if we're getting the full tensor data
            // Debug removed for cleaner output
            
            // Use the complete data vector - ensure no truncation
            let output = ArrayBase::from_shape_vec(IxDyn(&dims), data.to_vec())
                .expect("Failed to create array from tensor data");
            Ok(output)
        } else {
            Err("Session is not initialized.".into())
        }
    }
}
