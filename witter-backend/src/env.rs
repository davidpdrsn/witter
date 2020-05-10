#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Env {
    Production,
    Development,
    Test,
}

impl Env {
    pub fn is_test(&self) -> bool {
        match self {
            Env::Test => true,
            _ => false,
        }
    }
}

pub fn current() -> Env {
    match std::env::var("APP_ENV").unwrap().as_str() {
        "production" => Env::Production,
        "development" => Env::Development,
        "test" => Env::Test,
        _ => panic!(),
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;
    use crate::tests::test_helpers::*;

    #[async_std::test]
    async fn in_test_env_during_tests() {
        test_setup().await;

        let env = current();
        assert_eq!(env, Env::Test);
        assert!(env.is_test());
    }
}
