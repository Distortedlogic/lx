use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Company {
  pub id: String,
  pub name: String,
  pub issue_prefix: String,
}

#[derive(Clone, Copy)]
pub struct CompanyState {
  pub companies: Signal<Vec<Company>>,
  pub selected_id: Signal<Option<String>>,
}

impl CompanyState {
  pub fn provide() -> Self {
    let state = Self { companies: Signal::new(Vec::new()), selected_id: Signal::new(None) };
    use_context_provider(|| state);
    state
  }

  pub fn selected(&self) -> Option<Company> {
    let id = self.selected_id.read().clone();
    let companies = self.companies.read();
    id.and_then(|id| companies.iter().find(|c| c.id == id).cloned())
  }

  pub fn select(&self, company_id: String) {
    let mut s = self.selected_id;
    s.set(Some(company_id));
  }

  pub fn set_companies(&self, list: Vec<Company>) {
    let mut c = self.companies;
    c.set(list);
  }

  pub fn has_companies(&self) -> bool {
    !self.companies.read().is_empty()
  }
}
