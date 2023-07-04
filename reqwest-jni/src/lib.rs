use std::collections::HashMap;
use std::sync::OnceLock;

use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JMap, JObject, JString};
use jni::sys::jobject;
use once_cell::sync::Lazy;
use reqwest::{Client, Method, Url};
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());
static CLIENT: OnceLock<Client> = OnceLock::new();

#[no_mangle]
pub extern "system" fn Java_rocks_kavin_reqwest4j_ReqwestUtils_init(
    mut env: JNIEnv,
    _: JClass,
    proxy: JString,
) {
    let builder = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0");

    let builder = match env.get_string(&proxy) {
        Ok(proxy) => {
            let proxy = proxy.to_str().unwrap();
            let proxy = reqwest::Proxy::all(proxy).unwrap();
            builder.proxy(proxy)
        }
        Err(_) => {
            builder
        }
    };

    let client = builder.build()
        .unwrap();
    CLIENT.set(client).unwrap();
}

#[no_mangle]
pub extern "system" fn Java_rocks_kavin_reqwest4j_ReqwestUtils_fetch(
    mut env: JNIEnv,
    _: JClass,
    url: JString,
    method: JString,
    body: JByteArray,
    headers: JObject,
) -> jobject {

    // set method, url, body, headers
    let method = Method::from_bytes(env.get_string(&method).unwrap().to_bytes()).unwrap();

    let url = &env.get_string(&url).unwrap();
    let url = url.to_str();

    if url.is_err() {
        env.throw_new("java/lang/IllegalArgumentException", "Invalid URL provided, couldn't get string as UTF-8").unwrap();
        return JObject::null().into_raw();
    }

    let url = Url::parse(url.unwrap()).unwrap();
    let body = env.convert_byte_array(body).unwrap_or_default();
    let java_headers: JMap = JMap::from_env(&mut env, &headers).unwrap();
    let mut java_headers = java_headers.iter(&mut env).unwrap();
    let mut headers = HashMap::new();
    while let Some((key, value)) = java_headers.next(&mut env).unwrap() {
        headers.insert(
            env.get_string(&JString::from(key)).unwrap().to_str().unwrap().to_string(),
            env.get_string(&JString::from(value)).unwrap().to_str().unwrap().to_string(),
        );
    }

    let client = CLIENT.get();

    if client.is_none() {
        env.throw_new("java/lang/IllegalStateException", "Client not initialized").unwrap();
        return JObject::null().into_raw();
    }

    let client = client.unwrap();

    let request = client.request(method, url);

    let request = headers.into_iter().fold(request, |request, (key, value)| {
        request.header(key, value)
    });

    let request = if body.is_empty() {
        request
    } else {
        request.body(body)
    };

    // `JNIEnv` cannot be sent between threads safely
    let jvm = env.get_java_vm().unwrap();

    // create CompletableFuture
    let _future = env.new_object("java/util/concurrent/CompletableFuture", "()V", &[]).unwrap();
    let future = env.new_global_ref(&_future).unwrap();

    RUNTIME.spawn_blocking(move || {
        // send request
        let response = RUNTIME.block_on(async {
            request.send().await
        });

        let mut env = jvm.attach_current_thread().unwrap();

        if let Err(error) = response {
            let error = error.to_string();
            let error = env.new_string(error).unwrap();
            // create Exception
            let exception = env.new_object("java/lang/Exception", "(Ljava/lang/String;)V", &[
                (&error).into(),
            ]).unwrap();
            // pass error to CompletableFuture
            env.call_method(future, "completeExceptionally", "(Ljava/lang/Throwable;)Z", &[(&exception).into()]).unwrap();
            return ();
        }

        let response = response.unwrap();

        // get response
        let status = response.status().as_u16() as i32;

        let headers = env.new_object("java/util/HashMap", "()V", &[]).unwrap();
        let headers: JMap = JMap::from_env(&mut env, &headers).unwrap();

        response.headers().iter().for_each(|(key, value)| {
            let key = env.new_string(key.as_str()).unwrap();
            let value = env.new_string(value.to_str().unwrap()).unwrap();
            headers.put(&mut env, &JObject::from(key), &JObject::from(value)).unwrap();
        });

        let final_url = response.url().to_string();
        let final_url = env.new_string(final_url).unwrap();

        let body = RUNTIME.block_on(async {
            response.bytes().await.unwrap_or_default().to_vec()
        });

        let body = env.byte_array_from_slice(&body).unwrap();

        // return response to CompletableFuture
        let response = env.new_object("rocks/kavin/reqwest4j/Response", "(ILjava/util/Map;[BLjava/lang/String;)V", &[
            status.into(),
            (&headers).into(),
            (&body).into(),
            (&final_url).into(),
        ]).unwrap();

        let future = future.as_obj();
        env.call_method(future, "complete", "(Ljava/lang/Object;)Z", &[(&response).into()]).unwrap();
    });

    return _future.into_raw();
}
