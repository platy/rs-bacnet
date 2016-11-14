//! Implementations of messages sent to and from services

use ast::ValueSequence;

trait ServiceMessage where Self: Sized {
    fn unmarshall(body: &ValueSequence) -> Result<Self, UnmarshallError>;
    fn marshall(&self) -> ValueSequence;
}

#[derive(Debug, PartialEq)]
enum UnmarshallError {
    RequiredValueNotProvided,
}

mod whois {
    use ast::ValueSequence;
    use ast::PrimitiveValue::Unsigned;
    use ast::SequenceableValue::ContextValue;
    use ast::get_context_value;
    use super::ServiceMessage;
    use super::UnmarshallError;

    #[derive(Debug, PartialEq)]
    struct Message {
        device_instance_low: u32,
        device_instance_high: u32,
    }

    impl ServiceMessage for Message {
        fn marshall(&self) -> ValueSequence {
            vec!(
                ContextValue(0, Unsigned(self.device_instance_low)), 
                ContextValue(1, Unsigned(self.device_instance_high)))
        }

        fn unmarshall(body: &ValueSequence) -> Result<Self, UnmarshallError> {
            match (get_context_value(body, 0), get_context_value(body, 1)) {
                (Some(&Unsigned(low)), Some(&Unsigned(high))) => Ok(Message {
                    device_instance_low: low,
                    device_instance_high: high,
                }),
                _ => Err(UnmarshallError::RequiredValueNotProvided),
            }
        }
    }
    
    #[cfg(test)]
    mod test {
        use super::Message;
        use super::super::ServiceMessage;
        use ast::PrimitiveValue::Unsigned;
        use ast::SequenceableValue::ContextValue;

        #[test]
        fn test_unmarshall_correct() {
            assert_eq!(Ok(Message { device_instance_low: 1, device_instance_high: 50000 }),
                       Message::unmarshall(&vec!(
                              ContextValue(0, Unsigned(1)), 
                              ContextValue(1, Unsigned(50000)))));
        }

        #[test]
        fn test_marshall_correct() {
            assert_eq!(vec!(
                    ContextValue(0, Unsigned(1)),
                    ContextValue(1, Unsigned(50000))),
                    Message { device_instance_low: 1, device_instance_high: 50000 }.marshall());
        }

        #[test]
        fn test_marshall_cycle() {
            let message = Message { device_instance_low: 1, device_instance_high: 50000 };
            assert_eq!(message, Message::unmarshall(&message.marshall()).unwrap());
        }
    }
}

