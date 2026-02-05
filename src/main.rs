use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;
use serde::Deserialize;
use reqwasm::http::Request;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/logs")]
    Logs,
    #[at("/admin")]
    Admin,
    #[not_found]
    #[at("/404")]
    NotFound,
}

// ★修正点1: データの形をCMSに合わせて変更（URLを廃止し、画像と本文を追加）
#[derive(Clone, PartialEq, Deserialize)]
struct Post {
    id: usize,
    title: String,
    date: String,
    #[serde(default)]
    image: Option<String>, // 画像はない場合もあるのでOption
    #[serde(default)]
    body: Option<String>,  // 本文
}

#[derive(Clone, PartialEq, Deserialize)]
struct PostWrapper {
    posts: Vec<Post>,
}

// --- 以下、暗号解読機（変更なし） ---
#[derive(Clone, PartialEq)]
struct CipherResult {
    name: String,
    score: u32,
    breakdown: String,
}

struct GematriaDecoder {
    input_value: String,
    results: Vec<CipherResult>,
}

enum DecoderMsg {
    UpdateInput(String),
}

impl Component for GematriaDecoder {
    type Message = DecoderMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input_value: String::new(),
            results: Vec::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DecoderMsg::UpdateInput(val) => {
                if val.is_empty() {
                    self.input_value = val;
                    self.results.clear();
                    return true;
                }
                let mut std_sum = 0;
                let mut rev_sum = 0;
                let mut red_sum = 0;
                let mut std_parts = Vec::new();
                let mut rev_parts = Vec::new();
                for c in val.to_ascii_uppercase().chars() {
                    if c.is_ascii_alphabetic() {
                        let base_num = c as u32 - 64;
                        std_sum += base_num;
                        std_parts.push(base_num.to_string());
                        let rev_num = 27 - base_num;
                        rev_sum += rev_num;
                        rev_parts.push(rev_num.to_string());
                        let red_num = (base_num - 1) % 9 + 1;
                        red_sum += red_num;
                    }
                }
                self.results = vec![
                    CipherResult { name: "Standard".to_string(), score: std_sum, breakdown: std_parts.join("+") },
                    CipherResult { name: "Reverse".to_string(), score: rev_sum, breakdown: rev_parts.join("+") },
                    CipherResult { name: "Reduction".to_string(), score: red_sum, breakdown: "Reduced".to_string() },
                ];
                self.input_value = val;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            DecoderMsg::UpdateInput(input.value())
        });
        html! {
            <div>
                <section class="input-group">
                    <label>{ "> TARGET_IDENTIFIER: " }</label>
                    <input type="text" class="cmd-input" value={self.input_value.clone()} 
                           oninput={oninput} placeholder="Enter keyword..." autofocus=true />
                </section>
                if !self.results.is_empty() {
                    <section class="cipher-grid">
                        { for self.results.iter().map(|res| html! {
                            <div class="cipher-card">
                                <span class="cipher-label">{ &res.name }</span>
                                <span class="cipher-breakdown">{ &res.breakdown }</span>
                                <span class="cipher-value">{ res.score }</span>
                            </div>
                        }) }
                    </section>
                }
            </div>
        }
    }
}

// --- ★修正点2: 記事表示部分（Logs）をブログ風レイアウトに変更 ---
#[function_component(Logs)]
fn logs() -> Html {
    let posts = use_state(|| Vec::<Post>::new());
    
    {
        let posts = posts.clone();
        use_effect_with((), move |_| {
            let posts = posts.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // 万が一JSONの形が合わなくてもパニックしないようにエラーハンドリングを追加
                match Request::get("/posts.json").send().await {
                    Ok(resp) => {
                        if let Ok(fetched_wrapper) = resp.json::<PostWrapper>().await {
                            posts.set(fetched_wrapper.posts);
                        }
                    }
                    Err(_) => {} // エラー時は何もしない
                }
            });
            || ()
        });
    }

    html! {
        <section>
            <h2>{ "> SYSTEM_ARCHIVE" }</h2>
            
            <div style="display: flex; flex-direction: column; gap: 3rem;">
                { for posts.iter().map(|post| html! {
                    <article style="border: 1px solid #0f0; padding: 20px; border-radius: 4px;">
                        // 1. 日付とタイトル
                        <div style="border-bottom: 1px dashed #0f0; padding-bottom: 10px; margin-bottom: 15px;">
                            <span style="color: #888; font-size: 0.8rem;">{ &post.date }</span>
                            <h3 style="margin: 5px 0 0 0; font-size: 1.5rem;">{ &post.title }</h3>
                        </div>

                        // 2. 画像があれば表示（ここが見本サイトのようなアイキャッチ画像になります）
                        if let Some(img_url) = &post.image {
                            <div style="margin-bottom: 15px;">
                                <img src={img_url.clone()} style="max-width: 100%; height: auto; border: 1px solid #333;" />
                            </div>
                        }

                        // 3. 本文（改行を反映させる簡易表示）
                        if let Some(body_text) = &post.body {
                            <div style="white-space: pre-wrap; line-height: 1.6; color: #ddd;">
                                { body_text }
                            </div>
                        }
                    </article>
                }) }
            </div>

            if posts.is_empty() {
                <p style="color: #555;">{ "Scanning database..." }</p>
            }
        </section>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <GematriaDecoder /> },
        Route::Logs => html! { <Logs /> },
        Route::Admin => html! { <p>{ "Redirecting..." }</p> },
        Route::NotFound => html! { <h1>{ "404: SIGNAL LOST" }</h1> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <div class="terminal-wrapper">
                <header>
                    <h1 class="cursor-blink">{ "OCCULT CODE SYSTEM" }</h1>
                    <p class="status-line">
                        { "STATUS: ONLINE" } <span class="separator">{ "|" }</span>
                        { "USER: 418-User" } <span class="separator">{ "|" }</span>
                        { "NAV: ROUTING_ENABLED" }
                    </p>
                </header>
                <nav>
                    <ul>
                        <li><Link<Route> to={Route::Home}>{ "[H]ome" }</Link<Route>></li>
                        <li><Link<Route> to={Route::Logs}>{ "[L]ogs" }</Link<Route>></li>
                        <li><a href="/admin/">{ "[S]ystem_Admin" }</a></li>
                    </ul>
                </nav>
                <hr class="dashed-line" />
                <main>
                    <Switch<Route> render={switch} />
                </main>
                <footer>
                    <p>{ "END OF LINE." }</p>
                    <p>{ "© 2026 OCCULT CODE" }</p>
                </footer>
            </div>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}