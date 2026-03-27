use std::sync::Arc;

use crate::value::LxVal;

impl LxVal {
  pub fn bind_self(&self, self_val: &LxVal) -> LxVal {
    if let LxVal::Func(lf) = self {
      let env = lf.closure.child();
      env.bind_str("self", self_val.clone());
      let mut lf = lf.clone();
      lf.closure = Arc::new(env);
      LxVal::Func(lf)
    } else {
      self.clone()
    }
  }
}
