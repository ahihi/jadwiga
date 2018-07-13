use ::std::marker::PhantomData;
use ::std::str::FromStr;

use ::base64;
use ::openssl::{
    hash::MessageDigest,
    pkey::{PKey, Public},
    rsa::Rsa,
    sign::Verifier
};
use ::rocket::{
    outcome::Outcome,
    request::{
        self,
        FromRequest,
        Request,
    }
};
use ::serde_json::{self, Value};

use api::error::Error;
use parser;

#[derive(Debug)]
pub struct Signature {
    pub key_id: String,
    pub headers: Vec<String>,
    pub signature: Vec<u8>
}

impl FromStr for Signature {
    type Err = ::failure::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fields = parser::sig(s)?;

        let key_id = match fields.get("keyId") {
            Some(text) => text,
            None => return Err(format_err!("No 'keyId' field found"))
        };

        let headers = match fields.get("headers") {
            Some(text) => parser::sig_headers(text)?,
            None => return Err(format_err!("No 'headers' field found"))
        };

        let signature_b64 = match fields.get("signature") {
            Some(text) => text,
            None => return Err(format_err!("No 'signature' field found"))
        };

        let signature = base64::decode(&signature_b64)?;

        Ok(Signature {
            key_id: key_id.to_owned(),
            headers: headers,
            signature: signature
        })
    }
}

#[derive(Debug)]
pub struct ValidSignature {
    phantom: PhantomData<()>
}

fn get_valid_signature<'a, 'r>(request: &'a Request<'r>) -> Result<ValidSignature, ::failure::Error> {
    let headers = request.headers();

    println!("headers:\n");
    for header in headers.iter() {
        println!("{}: {}", header.name, header.value);
    }
    println!("");
    
    let signature_header = headers.get_one("Signature")
        .ok_or(format_err!("No 'Signature' header found"))?;

    let signature: Signature = signature_header.parse()
        .map_err(|e| format_err!("Failed to parse Signature: {:?}", e))?;

    println!("key_id: {:?}", signature.key_id);
    
    let actor_str: String = {
        use ::reqwest::{header, mime};

        let ld_json = "application/ld+json".parse::<mime::Mime>()?;
            
        ::reqwest::Client::new()
            .get(&signature.key_id)
            .header(header::Accept(vec![
                header::qitem(ld_json)
            ]))
            .send()
            .map_err(|e| format_err!("Failed to fetch actor: {:?}", e))?
            .text()
            .map_err(|e| format_err!("Failed to get actor body: {:?}", e))?
    };
    
    // println!("actor_str: {:?}", actor_str);

    let actor: Value = serde_json::from_str(&actor_str)
        .map_err(|e| format_err!("Failed to parse actor: {:?}", e))?;

    let public_key_pem_json: &Value = actor
        .get("publicKey")
        .ok_or(format_err!("No 'publicKey' field found on actor"))?
        .get("publicKeyPem")
        .ok_or(format_err!("No 'publicKeyPem' field found on public key"))?;

    let public_key_pem: &[u8] = match public_key_pem_json {
        Value::String(s) => s.as_bytes(),
        _ => return Err(format_err!("Value of 'publicKeyPem' is not a string"))
    };

    let public_key_rsa: Rsa<Public> = Rsa::public_key_from_pem(public_key_pem)
        .map_err(|e| format_err!("Failed to get RSA from PEM: {:?}", e))?;

    let public_key: PKey<Public> = PKey::from_rsa(public_key_rsa)
        .map_err(|e| format_err!("Failed to get public key from RSA: {:?}", e))?;
    
    let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key)
        .map_err(|e| format_err!("Failed to create key verifier: {:?}", e))?;

    let mut required_headers = signature.headers.clone();
    
    if required_headers.iter().find(|h| *h == "date").is_none() {
        required_headers.push("date".to_owned());
    }
    
    let comparison_string: String = required_headers.iter()
        .map(|header_name| {
            let header_name = header_name.to_lowercase();
            
            let value_opt = if header_name == "(request-target)" {
                let method = request.method().as_str().to_lowercase();
                let uri = request.uri().as_str();
                
                Ok(format!("{} {}", method, uri))
            } else {
                let values = headers.get(&header_name)
                    .map(|value| value.trim())
                    .collect::<Vec<&str>>();

                if values.len() > 0 {
                    Ok(values.join(", "))
                } else {
                    Err(format_err!("No '{}' header found", header_name))
                }
            };

            value_opt.map(|value| format!("{}: {}", header_name, value))
        })
        .collect::<Result<Vec<String>, _>>()?
        .join("\n");
    
    // println!("comparison_string: {:?}", comparison_string);
    
    verifier.update(comparison_string.as_bytes())?;

    let verified = verifier.verify(&signature.signature)?;
    
    if !verified {
        return Err(format_err!("Failed to verify signature"));
    }
    
    Ok(ValidSignature {phantom: PhantomData})
}

impl<'a, 'r> FromRequest<'a, 'r> for ValidSignature {
    type Error = Error;
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let result: Result<Self, Self::Error> = get_valid_signature(request)
            .map_err(Error::bad_request);

        println!("result: {:?}", result);

        match result {
            Ok(value) => Outcome::Success(value),
            Err(e) => Outcome::Failure((e.status(), e))
        }
    }
}
