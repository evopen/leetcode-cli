use self::req::{Json, Mode, Req};
use crate::{
    Result,
    config::{self, Config},
};
use reqwest::{
    Client, ClientBuilder, Response,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::{collections::HashMap, str::FromStr, time::Duration};

/// LeetCode API set
#[derive(Clone)]
pub struct LeetCode {
    pub conf: Config,
    client: Client,
    default_headers: HeaderMap,
}

impl LeetCode {
    /// Parse reqwest headers
    fn headers(mut headers: HeaderMap, ts: Vec<(&str, &str)>) -> Result<HeaderMap> {
        for (k, v) in ts.into_iter() {
            let name = HeaderName::from_str(k)?;
            let value = HeaderValue::from_str(v)?;
            headers.insert(name, value);
        }

        Ok(headers)
    }

    /// New LeetCode client
    pub fn new() -> Result<LeetCode> {
        let conf = config::Config::locate()?;
        let (cookie, csrf) = if conf.cookies.csrf.is_empty() || conf.cookies.session.is_empty() {
            let cookies = super::chrome::cookies()?;
            (cookies.to_string(), cookies.csrf)
        } else {
            (conf.cookies.clone().to_string(), conf.cookies.clone().csrf)
        };
        let default_headers = LeetCode::headers(
            HeaderMap::new(),
            vec![
                ("Cookie", &cookie),
                ("x-csrftoken", &csrf),
                ("x-requested-with", "XMLHttpRequest"),
                ("Origin", &conf.sys.urls.base),
            ],
        )?;

        let mut builder = ClientBuilder::new()
            .gzip(true)
            .connect_timeout(Duration::from_secs(30));
        if !conf.cookies.user_agent.is_empty() {
            builder = builder.user_agent(&conf.cookies.user_agent);
        }
        let client = builder.build()?;

        Ok(LeetCode {
            conf,
            client,
            default_headers,
        })
    }

    /// Get category problems
    pub async fn get_category_problems(self, category: &str) -> Result<Response> {
        trace!("Requesting {} problems...", category);
        let url = &self.conf.sys.urls.problems(category);

        Req {
            default_headers: self.default_headers,
            refer: None,
            info: false,
            json: None,
            mode: Mode::Get,
            name: "get_category_problems",
            url: url.to_string(),
        }
        .send(&self.client)
        .await
    }

    pub async fn get_question_ids_by_tag(self, slug: &str) -> Result<Response> {
        trace!("Requesting {} ref problems...", slug);
        let url = &self.conf.sys.urls.graphql;
        let mut json: Json = HashMap::new();
        json.insert("operationName", "getTopicTag".to_string());
        json.insert("variables", r#"{"slug": "$slug"}"#.replace("$slug", slug));
        json.insert(
            "query",
            [
                "query getTopicTag($slug: String!) {",
                "  topicTag(slug: $slug) {",
                "    questions {",
                "      questionId",
                "    }",
                "  }",
                "}",
            ]
            .join("\n"),
        );

        Req {
            default_headers: self.default_headers,
            refer: Some(self.conf.sys.urls.tag(slug)),
            info: false,
            json: Some(json),
            mode: Mode::Post,
            name: "get_question_ids_by_tag",
            url: (*url).to_string(),
        }
        .send(&self.client)
        .await
    }

    pub async fn get_user_info(self) -> Result<Response> {
        trace!("Requesting user info...");
        let url = &self.conf.sys.urls.graphql;
        let mut json: Json = HashMap::new();
        json.insert("operationName", "a".to_string());
        json.insert(
            "query",
            "query a {
                 user {
                     username
                     isCurrentUserPremium
                 }
             }"
            .to_owned(),
        );

        Req {
            default_headers: self.default_headers,
            refer: None,
            info: false,
            json: Some(json),
            mode: Mode::Post,
            name: "get_user_info",
            url: (*url).to_string(),
        }
        .send(&self.client)
        .await
    }

    /// Get daily problem
    pub async fn get_question_daily(self) -> Result<Response> {
        trace!("Requesting daily problem...");
        let url = &self.conf.sys.urls.graphql;
        let mut json: Json = HashMap::new();

        match self.conf.cookies.site {
            config::LeetcodeSite::LeetcodeCom => {
                json.insert("operationName", "daily".to_string());
                json.insert(
                    "query",
                    [
                        "query daily {",
                        "  activeDailyCodingChallengeQuestion {",
                        "    question {",
                        "      questionFrontendId",
                        "    }",
                        "  }",
                        "}",
                    ]
                    .join("\n"),
                );
            }
            config::LeetcodeSite::LeetcodeCn => {
                json.insert("operationName", "questionOfToday".to_string());
                json.insert(
                    "query",
                    [
                        "query questionOfToday {",
                        "  todayRecord {",
                        "    question {",
                        "      questionFrontendId",
                        "    }",
                        "  }",
                        "}",
                    ]
                    .join("\n"),
                );
            }
        }

        Req {
            default_headers: self.default_headers,
            refer: None,
            info: false,
            json: Some(json),
            mode: Mode::Post,
            name: "get_question_daily",
            url: (*url).to_string(),
        }
        .send(&self.client)
        .await
    }

    /// Get specific problem detail
    pub async fn get_question_detail(self, slug: &str) -> Result<Response> {
        trace!("Requesting {} detail...", slug);
        let refer = self.conf.sys.urls.problem(slug);
        let mut json: Json = HashMap::new();
        json.insert(
            "query",
            [
                "query getQuestionDetail($titleSlug: String!) {",
                "  question(titleSlug: $titleSlug) {",
                "    content",
                "    stats",
                "    codeDefinition",
                "    sampleTestCase",
                "    exampleTestcases",
                "    enableRunCode",
                "    metaData",
                "    translatedContent",
                "  }",
                "}",
            ]
            .join("\n"),
        );

        json.insert(
            "variables",
            r#"{"titleSlug": "$titleSlug"}"#.replace("$titleSlug", slug),
        );

        json.insert("operationName", "getQuestionDetail".to_string());

        Req {
            default_headers: self.default_headers,
            refer: Some(refer),
            info: false,
            json: Some(json),
            mode: Mode::Post,
            name: "get_problem_detail",
            url: self.conf.sys.urls.graphql,
        }
        .send(&self.client)
        .await
    }

    /// Send code to judge
    pub async fn run_code(self, j: Json, url: String, refer: String) -> Result<Response> {
        info!("Sending code to judge...");
        Req {
            default_headers: self.default_headers,
            refer: Some(refer),
            info: false,
            json: Some(j),
            mode: Mode::Post,
            name: "run_code",
            url,
        }
        .send(&self.client)
        .await
    }

    /// Get the result of submission / testing
    pub async fn verify_result(self, id: String) -> Result<Response> {
        trace!("Verifying result...");
        let url = self.conf.sys.urls.verify(&id);

        Req {
            default_headers: self.default_headers,
            refer: None,
            info: false,
            json: None,
            mode: Mode::Get,
            name: "verify_result",
            url,
        }
        .send(&self.client)
        .await
    }
}

/// Sub-module for leetcode, simplify requests
mod req {
    use super::LeetCode;
    use crate::err::Error;
    use reqwest::{Client, Response, header::HeaderMap};
    use std::collections::HashMap;

    /// Standardize json format
    pub type Json = HashMap<&'static str, String>;

    /// Standardize request mode
    pub enum Mode {
        Get,
        Post,
    }

    /// LeetCode request prototype
    pub struct Req {
        pub default_headers: HeaderMap,
        pub refer: Option<String>,
        pub json: Option<Json>,
        pub info: bool,
        pub mode: Mode,
        pub name: &'static str,
        pub url: String,
    }

    impl Req {
        pub async fn send(self, client: &Client) -> Result<Response, Error> {
            trace!("Running leetcode::{}...", self.name);
            if self.info {
                info!("{}", self.name);
            }
            let url = self.url.to_owned();
            let headers = LeetCode::headers(
                self.default_headers,
                vec![("Referer", &self.refer.unwrap_or(url))],
            )?;

            let req = match self.mode {
                Mode::Get => client.get(&self.url),
                Mode::Post => client.post(&self.url).json(&self.json),
            };

            let response = req.headers(headers).send().await?;
            debug!(
                "Response protocol: {:?}, status: {}",
                response.version(),
                response.status()
            );
            if response
                .headers()
                .get("cf-mitigated")
                .is_some_and(|v| v == "challenge")
            {
                return Err(anyhow::anyhow!(
                    "Cloudflare challenge blocked {} (HTTP {}). Open LeetCode in Chrome, complete any verification, and configure the matching cookies.user_agent and cf_clearance (automatically read with Chrome login cookies). Use the same network connection as Chrome. Clearance reuse is not guaranteed.",
                    self.name, response.status()
                ).into());
            }
            Ok(response)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        };

        #[tokio::test]
        async fn sends_browser_headers_and_detects_challenges() {
            for challenged in [false, true] {
                let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let url = format!("http://{}", listener.local_addr().unwrap());
                let server = tokio::spawn(async move {
                    let (mut socket, _) = listener.accept().await.unwrap();
                    let mut request = Vec::new();
                    loop {
                        let mut buffer = [0; 1024];
                        let count = socket.read(&mut buffer).await.unwrap();
                        assert!(count > 0);
                        request.extend_from_slice(&buffer[..count]);
                        if request.windows(4).any(|v| v == b"\r\n\r\n") {
                            break;
                        }
                    }
                    let request = String::from_utf8(request).unwrap().to_lowercase();
                    assert!(request.contains("user-agent: test-browser"));
                    assert!(request.contains("cookie: cf_clearance=test-clearance;"));
                    let response = if challenged {
                        "HTTP/1.1 403 Forbidden\r\ncf-mitigated: challenge\r\nContent-Length: 4\r\nConnection: close\r\n\r\nhtml"
                    } else {
                        "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}"
                    };
                    socket.write_all(response.as_bytes()).await.unwrap();
                });
                let client = Client::builder()
                    .timeout(std::time::Duration::from_secs(5))
                    .no_proxy()
                    .user_agent("test-browser")
                    .build()
                    .unwrap();
                let mut headers = HeaderMap::new();
                headers.insert("Cookie", "cf_clearance=test-clearance;".parse().unwrap());
                let result = Req {
                    default_headers: headers,
                    refer: None,
                    json: None,
                    info: false,
                    mode: Mode::Get,
                    name: "test",
                    url,
                }
                .send(&client)
                .await;
                if challenged {
                    let error = format!("{:?}", result.unwrap_err());
                    assert!(error.contains("Cloudflare challenge"));
                    assert!(!error.contains("html"));
                } else {
                    assert_eq!(result.unwrap().text().await.unwrap(), "{}");
                }
                server.await.unwrap();
            }
        }
    }
}
