use std::collections::HashMap;

use jni::JNIEnv;
use jni::objects::{JClass, JMap, JObject, JString};
use jni::sys::{jbyteArray, jobject};
use lazy_static::lazy_static;
use reqwest::{Client, Method, Url};
use tokio::runtime::Runtime;

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref CLIENT: Client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0")
        .build()
        .unwrap();
}

#[no_mangle]
pub extern "system" fn Java_rocks_kavin_reqwest4j_ReqwestUtils_fetch(
    env: JNIEnv,
    _: JClass,
    url: JString,
    method: JString,
    body: jbyteArray,
    headers: JObject,
) -> jobject {

    // set method, url, body, headers
    let method = Method::from_bytes(env.get_string(method).unwrap().to_bytes()).unwrap();

    let url = &env.get_string(url).unwrap();
    let url = url.to_str();

    if url.is_err() {
        env.throw_new("java/lang/IllegalArgumentException", "Invalid URL provided, couldn't get string as UTF-8").unwrap();
        return JObject::null().into_raw();
    }

    let url = Url::parse(url.unwrap()).unwrap();
    let body = env.convert_byte_array(body).unwrap_or_default();
    let headers: JMap = JMap::from_env(&env, headers).unwrap();
    let headers = headers.iter().unwrap().fold(HashMap::new(), |mut headers, (key, value)| {
        headers.insert(
            env.get_string(JString::from(key)).unwrap().to_str().unwrap().to_string(),
            env.get_string(JString::from(value)).unwrap().to_str().unwrap().to_string(),
        );
        headers
    });

    let request = CLIENT.request(method, url);

    let request = headers.into_iter().fold(request, |request, (key, value)| {
        request.header(key, value)
    });

    let request = if body.is_empty() {
        request
    } else {
        request.body(body)
    };

    // send request
    let response = RUNTIME.block_on(async {
        request.send().await.unwrap()
    });

    // get response
    let status = response.status().as_u16() as i32;

    let headers = env.new_object("java/util/HashMap", "()V", &[]).unwrap();
    let headers: JMap = JMap::from_env(&env, headers).unwrap();

    response.headers().iter().for_each(|(key, value)| {
        let key = env.new_string(key.as_str()).unwrap();
        let value = env.new_string(value.to_str().unwrap()).unwrap();
        headers.put(JObject::from(key), JObject::from(value)).unwrap();
    });

    let final_url = response.url().to_string();
    let final_url = env.new_string(final_url).unwrap();

    let body = RUNTIME.block_on(async {
        response.bytes().await.unwrap_or_default().to_vec()
    });


    let body = env.byte_array_from_slice(&body).unwrap();
    let body = unsafe { JObject::from_raw(body) };

    // return response
    let response = env.new_object("rocks/kavin/reqwest4j/Response", "(ILjava/util/Map;[BLjava/lang/String;)V", &[
        status.into(),
        headers.into(),
        body.into(),
        final_url.into(),
    ]).unwrap();

    response.into_raw()
}
