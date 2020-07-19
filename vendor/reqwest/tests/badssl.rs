#[cfg(feature = "__tls")]
#[tokio::test]
async fn test_badssl_modern() {
    let text = reqwest::get("https://mozilla-modern.badssl.com/")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(text.contains("<title>mozilla-modern.badssl.com</title>"));
}

#[cfg(feature = "rustls-tls")]
#[tokio::test]
async fn test_rustls_badssl_modern() {
    let text = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .unwrap()
        .get("https://mozilla-modern.badssl.com/")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(text.contains("<title>mozilla-modern.badssl.com</title>"));
}

#[cfg(feature = "__tls")]
#[tokio::test]
async fn test_badssl_self_signed() {
    let text = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .get("https://self-signed.badssl.com/")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(text.contains("<title>self-signed.badssl.com</title>"));
}

#[cfg(feature = "native-tls")]
#[tokio::test]
async fn test_badssl_wrong_host() {
    let text = reqwest::Client::builder()
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap()
        .get("https://wrong.host.badssl.com/")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(text.contains("<title>wrong.host.badssl.com</title>"));

    let result = reqwest::Client::builder()
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap()
        .get("https://self-signed.badssl.com/")
        .send()
        .await;

    assert!(result.is_err());
}
