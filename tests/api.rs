extern crate exonum_try;
extern crate yukikaze;
#[macro_use]
extern crate serde_derive;

use yukikaze::rt::{AutoClient, AutoRuntime};
use yukikaze::client::Request;

const API: &'static str = "http://127.0.0.1:8000/api/services/auction/v1";
const SIGNATURE: &'static str = "9f684227f1de663775848b3db656bca685e085391e2b00b0e115679fd45443ef58a5abeb555ab3d5f7a3cd27955a2079e5fd486743f36515c8e5bea07992100b";

//Tests assume that server runs

static INIT: std::sync::Once = std::sync::Once::new();

fn init_client() -> yukikaze::rt::Guard {
    use yukikaze::rt;

    INIT.call_once(|| {
        rt::set_default()
    });

    rt::init()
}

#[derive(Serialize)]
struct NewArticle<'a> {
    pub_key: &'a str,
    name: &'a str
}
#[derive(Serialize)]
struct CreateArticle<'a> {
    body: NewArticle<'a>,
    protocol_version: u8,
    service_id: u16,
    message_id: u32,
    signature: &'a str,
}

impl<'a> CreateArticle<'a> {
    fn new(pub_key: &'a str, name: &'a str) -> Self {
        let body = NewArticle {
            pub_key,
            name
        };

        Self {
            body,
            protocol_version: 0,
            service_id: exonum_try::block::article::SERVICE_ID,
            message_id: 0,
            signature: SIGNATURE
        }
    }
}

#[test]
fn it_works() {
    let _guard = init_client();
    let new_article = CreateArticle::new("6ce29b2d3ecadc434107ce52c287001c968a1b6eca3e5a1eb62a2419e2924b85", "Frist lot");
    let result = Request::post(format!("{}/article/new", API)).expect("To create request")
                                                              .json(&new_article)
                                                              .expect("To serialize json")
                                                              .send()
                                                              .finish()
                                                              .expect("Successfully post article");

    //TODO: I guess I'd need to look into actual clients for exonum framework
    println!("{:?}", result);
    assert!(result.is_success());
}
