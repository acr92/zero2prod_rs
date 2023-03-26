use std::convert::TryFrom;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(email: String) -> Result<Self, Self::Error> {
        if validator::validate_email(&email) {
            Ok(Self(email))
        } else {
            Err(format!("{} is not a valid email", email))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
pub mod tests {
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::Gen;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use crate::domain::SubscriberEmail;

    #[test]
    fn invalid_email() {
        let invalid_emails = vec!["", "a", "a@", "@a"];
        for invalid_email in invalid_emails {
            assert_err!(SubscriberEmail::try_from(invalid_email.to_string()));
        }
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_email(valid_email: ValidEmailFixture) {
        assert_ok!(SubscriberEmail::try_from(valid_email.0));
    }
}
