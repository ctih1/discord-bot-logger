use rouille;
use rouille::router;
use serenity::all::Route;
use std::collections::HashMap;
use std::fs;
use std::env;
use crate::database;
use crate::futures::executor;
use chrono::prelude::{DateTime};

struct DatabaseTarget {
    id: u64,
    user_id: u64,
    time: u64,
    status: String,
    activity: String,
    activity_description: String,
}

#[allow(unreachable_code)]

pub fn main(key: String) {
    let key: String = key;
    println!("Now listening on 0.0.0.0:8000");

    rouille::start_server("0.0.0.0:8000", move |request| {
        router!(request,
            (GET) (/) => {
                let cookies = parse_cookies(request);

                if let Some(auth_token) = cookies.get("Authorization") {
                    if auth_token != &key {
                        return rouille::Response::redirect_302("/login#invalid");
                    } 
                } else {
                    return rouille::Response::redirect_302("/login#not-specified");
                }
                
                
                
                let page_number: u64 = cookies.get("page").unwrap_or(&String::from("1")).parse().unwrap();

                let data = executor::block_on(retrieve_data_from_db(
                    page_number,
                    cookies.get("userId").map(|x| x.as_str()), 
                    cookies.get("status").map(|x| x.as_str()),
                    cookies.get("activity").map(|x| x.as_str()), 
                    cookies.get("activity_description").map(|x| x.as_str()),
                    None, 
                    None
                    ));
    
                return rouille::Response::html(construct_page(data, page_number, 500));
            },

            (GET) (/login) => {
                let cookies = parse_cookies(request);

                if let Some(auth_token) = cookies.get("token") {
                    if auth_token == &key {
                        return rouille::Response::redirect_302("/")
                        .with_additional_header("Set-Cookie", format!("Authorization={auth_token}; max-age=10800; HttpOnly"));
                    } 
                }

                return rouille::Response::html(
                    "
                    <head>
                        <link rel=\"stylesheet\" href=\"https://unpkg.com/@picocss/pico@latest/css/pico.min.css\" />

                        <style>
                            body {
                                padding: 2em;
                            }
                        </style>
                    </head>
                    <body>
                        <label>Enter access token. You can retrieve it by running /login</label>
                        <input id=\"token\" placeholder=\"Token\">
                        <button onclick=\"login()\">login</button>
                    </body>
                
                    <script>
                        function login() {
                            const token = document.getElementById('token').value;

                            document.cookie = `token=${token}`;
                            window.location.reload();
                        }
                    </script>
                    "
                );
            },

            _ => rouille::Response::empty_404()
        )
    });
}

async fn retrieve_data_from_db(page: u64, user_id: Option<&str>, status: Option<&str>, activity: Option<&str>, activity_description: Option<&str>, time_lt: Option<&u64>, time_mt: Option<&u64>) -> Vec<DatabaseTarget> {
    let mut retrieved_data: Vec<DatabaseTarget> = vec!();

    let mut rows: libsql::Rows = database::get_data(&page,user_id, status, activity, activity_description, time_lt, time_mt).await;

    while let Ok(Some(row)) = rows.next().await {
        let id: u64 = row.get(0).unwrap();
        let user_id: u64 = row.get(1).unwrap();
        let time: u64 = row.get(2).unwrap();
        let status: String = row.get(3).unwrap();
        let activity: String = row.get(4).unwrap();
        let activity_description: String = row.get(5).unwrap();

        retrieved_data.push(DatabaseTarget { id: id, user_id: user_id, time: time, status: status, activity: activity, activity_description: activity_description });
    }

    return retrieved_data;
}


fn construct_results(data: Vec<DatabaseTarget>) -> String {
    let mut html_string: String = String::from("");
    let usernames = executor::block_on(database::get_usernames(data.iter().map(|s| s.user_id).collect()));


    for result in data.iter() {
        let target_username = usernames.get(&result.user_id).unwrap();
        let time: i64 = result.time.try_into().unwrap();

        let readable_time = DateTime::from_timestamp(time,0).unwrap().format("%d/%m/%Y @ %H:%M:%S");

        html_string += format!("
        <article class=\"status {}\">
            <h3><span data-userid=\"{}\"  class=\"mention\">{}</span></h3>
            <h4>{}</h4>
            <hr>
            <h5>{} - {}</h5>
        </article>
      ", result.status, result.user_id, target_username, format!("{}", readable_time), result.activity, result.activity_description)
            .as_str();
    }

    return html_string;
}

fn construct_page(data: Vec<DatabaseTarget>, page: u64, max_pages: u64) -> String {
    let html = format!(
    "
<html>
<head>
    <link rel=\"stylesheet\" href=\"https://unpkg.com/@picocss/pico@latest/css/pico.min.css\" />

    <style>
    :root {{
    --pico-font-family-sans-serif: Inter, system-ui, \"Segoe UI\", Roboto, Oxygen, Ubuntu, Cantarell, Helvetica, Arial, \"Helvetica Neue\", sans-serif, var(--pico-font-family-emoji);
    --pico-font-size: 87.5%;
    /* Original: 100% */
    --pico-line-height: 1.25;
    /* Original: 1.5 */
    --pico-form-element-spacing-vertical: 0.5rem;
    /* Original: 1rem */
    --pico-form-element-spacing-horizontal: 1.0rem;
    /* Original: 1.25rem */
    --pico-border-radius: 0.375rem;
    /* Original: 0.25rem */
}}

@media (min-width: 576px) {{
    :root {{
        --pico-font-size: 87.5%;
        /* Original: 106.25% */
    }}
}}

@media (min-width: 768px) {{
    :root {{
        --pico-font-size: 87.5%;
        /* Original: 112.5% */
    }}
}}

@media (min-width: 1024px) {{
    :root {{
        --pico-font-size: 87.5%;
        /* Original: 118.75% */
    }}
}}

@media (min-width: 1280px) {{
    :root {{
        --pico-font-size: 87.5%;
        /* Original: 125% */
    }}
}}

@media (min-width: 1536px) {{
    :root {{
        --pico-font-size: 87.5%;
        /* Original: 131.25% */
    }}
}}

h1,
h2,
h3,
h4,
h5,
h6 {{
    --pico-font-weight: 600;
    /* Original: 700 */
}}

article {{
    border: 1px solid var(--pico-muted-border-color);
    /* Original doesn't have a border */
    border-radius: calc(var(--pico-border-radius) * 2);
    /* Original: var(--pico-border-radius) */
}}

article>footer {{
    border-radius: calc(var(--pico-border-radius) * 2);
    /* Original: var(--pico-border-radius) */
}}

        body {{
            padding: 2em;
        }}

        .navigation {{
            display: flex;
            align-items: center;
        }}

        .filters {{
            display: flex;
            justify-content: center;
            flex-direction: column;
        }}

        .horizontal {{
            display: flex;
            align-items: center;
            justify-content: center;
            flex-direction: row;
        }}

        .filters input {{}}

        .navigation * {{
            width: fit-content;
            height: fit-content;
            align-items: center;
        }}

        .navigation button {{
            margin: 1em;
        }}

        .statuses div {{
            min-width: 50vw;
            width: fit-content;
            margin-bottom: 12px;
        }}

        #pages * {{
            height: 1em;
        }}

        .status {{
            border: solid 0px;
            border-left-width: 10px !important;
        }}

        .online {{
            border-color: rgb(40, 219, 37) !important;
        }}

        .idle {{
            border-color: rgb(219, 157, 24) !important;
        }}

        .dnd {{
            border-color: rgb(230, 54, 41) !important;
        }}

        .offline {{
            border-color: rgb(53, 56, 59) !important;
        }}

        
        .mention {{
            background-color: #3e4270;
            padding: 6px;
            border-radius: 4px;
        }}
        
        .mention::before {{
            content: \"@\";
        }}

        
    </style>

</head>

<body>
    <h1>Status</h1>
    <div class=\"filters\">
        <p>Filters</p>
        <div class=\"horizontal-filters\">
            <input id=\"id\" placeholder=\"User ID\">
            <input id=\"activity\" placeholder=\"Activity (e.g Spotify)\">
            <label>Status</label>
            <select id=\"status\">
                <option value=\"\" selected>Any</option>
                <option value=\"online\">Online</option>
                <option value=\"dnd\">Do not disturb</option>
                <option value=\"idle\">Idle</option>
                <option value=\"offline\">Offline</option>
            </select>
        </div>
        <div class=\"horizontal-filters\">
            <div>
                <label>Before:</label>
                <input id=\"before\" type=\"date\" placeholder=\"before (e.g 1745738385)\">
            </div>
            <div>
                <label>After:</label>
                <input id=\"after\" type=\"date\" placeholder=\"after (e.g 1745738385\">
            </div>
        </div>
    </div>
    <button onclick=\"handleFilterApply();\">Apply</button>

    <hr>
    </div>

    <div class=\"statuses\">
        {}
    </div>

    <div class=\"navigation horizontal\">
        <button class=\"outline\" onclick=\"handleNavigation(-1)\">Previous page</button>
        <div id=\"pages\" class=\"horizontal\">
            <p id=\"selected-page\">{page}</p>
            <p id=\"max-pages\">/{max_pages}</p>
        </div>
        <button class=\"outline\" onclick=\"handleNavigation(1)\">Next page</button>
    </div>

    <script>
        const userId = document.getElementById('id');
        const activity = document.getElementById('activity');
        const status = document.getElementById('status');
        userId.value = getCookieByName('userId');
        activity.value = getCookieByName('activity');
        status.value = getCookieByName('status');
        document.cookie = \"token=no;expires=Thu, 01 Jan 1970 00:00:01 GMT\";

        function getCookieByName(name) {{
            const cookies = document.cookie.split(';');
            for (let cookie of cookies) {{
                cookie = cookie.trim();
                if (cookie.startsWith(name + '=')) {{
                    return cookie.substring(name.length + 1);
                }}
            }}
            return null;
        }}

        function eraseCookie(name) {{
            document.cookie = name + '=; Max-Age=0'
        }}

        function handleNavigation(direction) {{
            let currentPage = Number(getCookieByName(\"page\"));
            currentPage += direction;
            if (currentPage <= 0) {{
                return;
            }}
            document.cookie = \"page=\" + currentPage;
            window.location.reload();
        }}

        function handleFilterApply(a) {{
            if (activity.value) {{
                document.cookie = \"activity=\" + activity.value;
            }} else {{
                eraseCookie(\"activity\");
            }}
            if (userId.value) {{
                document.cookie = \"userId=\" + userId.value;
            }} else {{
                eraseCookie(\"userId\");
            }}
            if (status.value) {{
                document.cookie = \"status=\" + status.value;
            }} else {{
                eraseCookie(\"status\");
            }}
            console.log(\"id=\" + userId.value + \"; activity=\" + activity.value + \"; status=\" + status.value);
            window.location.reload();
        }}
    </script>
</body>

</html>
", construct_results(data));

  return html;
}

fn parse_cookies(request: &rouille::Request) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    for (header, value) in request.headers() {
        if header.to_lowercase() == "cookie" {
            for cookie in value.split(';') {
                if let Some((name, val)) = cookie.trim().split_once('=') {
                    cookies.insert(name.trim().to_string(), val.trim().to_string());
                }
            }
        }
    }
    return cookies;
}
