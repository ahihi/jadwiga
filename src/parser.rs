use ::std::collections::HashMap;

// Signature:
// keyId="https://my-example.com/actor#main-key",headers="(request-target) host date",signature="Y2FiYW...IxNGRiZDk4ZA=="

pub fn sig(text: &str) -> Result<HashMap<String, String>, ::failure::Error> {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    enum State {
        Key,
        PreValue,
        Value,
        PostValue
    }
    
    let mut map = HashMap::new();
    let mut state = State::Key;
    let mut key = String::new();
    let mut value = String::new();

    let err = |state: State, ch: char| Err(format_err!("Signature parsing error: state = {:?}, ch = {:?}", state, ch));
    
    for ch in text.chars() {
        match (state, ch) {
            (State::Key, '=') => {
                state = State::PreValue
            },
            (State::Key, _) => {
                key.push(ch);
            },
            (State::PreValue, '"') => {
                state = State::Value;
            },
            (State::PreValue, _) => {
                return err(state, ch);
            },
            (State::Value, '"') => {
                map.insert(key.clone(), value.clone());
                state = State::PostValue
            },
            (State::Value, _) => {
                value.push(ch);
            },
            (State::PostValue, ',') => {
                key.clear();
                value.clear();
                state = State::Key;
            },
            (State::PostValue, _) => {
                return err(state, ch);
            }
        }
    }

    if state != State::PostValue {
        return Err(format_err!("Reached EOF prematurely: state = {:?}", state));
    }
    
    Ok(map)
}

pub fn sig_headers(text: &str) -> Result<Vec<String>, ::failure::Error> {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    enum State {
        Whitespace,
        Header
    }        
    
    let mut headers = vec![];
    let mut state = State::Whitespace;
    let mut header = String::new();
    
    for ch in text.chars() {
        match (state, ch) {
            (State::Whitespace, ' ') => {},
            (State::Whitespace, _) => {
                header.push(ch);
                state = State::Header;
            },
            (State::Header, ' ') => {
                headers.push(header.clone());
                header.clear();
                state = State::Whitespace;
            },
            (State::Header, _) => {
                header.push(ch);
            }
        }
    }

    if state == State::Header {
        headers.push(header);
    }
    
    Ok(headers)
}
