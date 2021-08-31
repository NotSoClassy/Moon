use crate::common::Type;
use crate::vm::CallInfo;

pub enum RuntimeError {
  TypeError(String, Type, Option<Type>),
  CustomError(String),
  StackOverflow,
  ArrayIdxBound,
  ArrayIdxFloat,
  ArrayIdxNeg
}

impl RuntimeError {
  pub fn to_error(&self, call_stack: &Vec<CallInfo>) -> String {
    let err = self.stringify();
    let trace = self.trace(call_stack);

    format!("{}\n{}", err, trace)
  }

  fn trace(&self, call_stack: &Vec<CallInfo>) -> String {
    let mut trace = "stack trace:\n".to_string();

    if let RuntimeError::StackOverflow = self {
      let top = call_stack.last().unwrap(); // I don't think this should error
      let overflowed = self.fmt_trace(top);

      trace += format!("{}\t...\n{}", overflowed, overflowed).as_str();

      for idx in (0 .. call_stack.len()).rev() {
        let call = &call_stack[idx];

        if call.closure != top.closure {
          trace += self.fmt_trace(call).as_str()
        }
      }
    } else {
      for idx in (0 .. call_stack.len()).rev() {
        let call = &call_stack[idx];

        trace += self.fmt_trace(call).as_str()
      }
    }

    trace
  }

  fn fmt_trace(&self, info: &CallInfo) -> String {
    if info.is_builtin {
      format!("\t[{}] in function {}\n", info.closure.file_name, info.closure.name)
    } else {
      format!("\t[{}] in function {}\n", info.closure.file_name, info.closure.name)
    }
  }

  fn stringify(&self) -> String {
    match self {
      RuntimeError::TypeError(err, t1, t2) => {
        if let Some(t2) = t2 {
          format!("attempt to {} on a {:?} and {:?} value", err, t1, t2)
        } else {
          format!("attempt to {} on a {:?} value", err, t1)
        }
      },

      RuntimeError::CustomError(err) => err.into(),
      RuntimeError::StackOverflow => "stack overflow".into(),
      RuntimeError::ArrayIdxBound => "array index out of bounds".into(),
      RuntimeError::ArrayIdxFloat => "array index must be an integer".into(),
      RuntimeError::ArrayIdxNeg => "array index must be positive".into()
    }
  }
}