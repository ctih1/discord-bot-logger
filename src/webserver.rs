use rouille;
use rouille::router;
use serenity::all::Route;
use std::collections::HashMap;
use std::fs;
use std::env;
use crate::database;
use crate::futures::executor;

struct DatabaseTarget {
    id: u64,
    user_id: u64,
    time: u64,
    status: String,
    activity: String,
    activity_description: String,
}

#[allow(unreachable_code)]

pub fn main() {
    println!("Now listening on localhost:8000");

    rouille::start_server("localhost:8000", move |request| {
        router!(request,
            (GET) (/) => {
                let cookies = parse_cookies(request);

                let mut key_is_defined = true;

                env::var("key").unwrap_or_else(|_| {
                    key_is_defined = false;
                    return "".to_string();
                });

                if !key_is_defined {
                    println!("Key is not defined in env. Redirecting to login");
                    return rouille::Response::redirect_302("/login#not-defined");
                }
                
                if let Some(auth_token) = cookies.get("Authorization") {
                    if auth_token != &env::var("key").unwrap() {
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
                
                let mut key_is_defined = true;

                let key = env::var("key").unwrap_or_else(|_| {
                    key_is_defined = false;
                    return "".to_string();
                });

                
                if !key_is_defined {
                    return rouille::Response::html("<h1>Login key not defined</h1> <p>Execute /login on discord to get a key</p>");
                }

                if let Some(auth_token) = cookies.get("token") {
                    if auth_token == &key {
                        return rouille::Response::redirect_302("/")
                        .with_additional_header("Set-Cookie", format!("Authorization={auth_token}; max-age=10800; HttpOnly"));
                    } 
                }

                return rouille::Response::html(
                    "<input id=\"token\" placeholder=\"token\">
                    <button onclick=\"login()\">login</button>

                
                    <script>
                        function login() {
                            const token = document.getElementById('token').value;

                            document.cookie = `token=${token}; max-age=60`;
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

    for result in data.iter() {
        html_string += format!("
        <div class={}>
            <p>{}</p>
            <p>{}</p>
            <p>{}</p>
            <p>{}</p>
            <p>{}</p>
        </div>
      ", result.status, result.time, result.user_id, result.status, result.activity, result.activity_description)
            .as_str();
    }

    return html_string;
}

fn construct_page(data: Vec<DatabaseTarget>, page: u64, max_pages: u64) -> String {
    let html = format!(
    "<style>
        .navigation {{
        display: flex;
        align-items: center;
    }}
    
    .navigation * {{
        width: fit-content;
        height: fit-content;
        margin: 0.5em;
    }}

    .navigation input {{
        width: 8ch;
    }}
    
    .statuses div {{
        background-color: gray;
        min-width: 50vw;
        width: fit-content;
        margin-bottom: 12px;
    }}

    .online {{
        background-color: green !important;
    }}
      
    .offline {{
       background-color: gray !important;
    }}
  
    .dnd {{
      background-color: red !important;
    }}
  
    .idle {{
      background-color: orange !important;
    }}
  </style>
  
  <body>
    <h1>Status</h1>
    <div class=\"filters\">
      <input id=\"id\" placeholder=\"user id\">
      <input id=\"activity\" placeholder=\"activity (e.g spotify)\">
      <input id=\"status\" placeholder=\"status (e.g online)\">
      <input id=\"before\" type=\"date\" placeholder=\"before (e.g 1745738385)\">
      <input id=\"after\" type=\"date\"  placeholder=\"after (e.g 1745738385\">
      <button onclick=\"handleFilterApply();\">apply</button>
    </div>
    <div class=\"navigation\">
      <button onclick=\"handleNavigation(-1)\">previous page</button>
      <input value={page}>
      <p id=\"pagenum\">/{max_pages}</p>
      <button onclick=\"handleNavigation(1)\">next page</button>
    </div>
    <div class=\"statuses\">  
        {}
    </div>
  </body>
      
  <script>
    const userId = document.getElementById('id');
    const activity = document.getElementById('activity');
    const status = document.getElementById('status');

    userId.value = getCookieByName('userId');
    activity.value = getCookieByName('activity');
    status.value = getCookieByName('status');

    document.cookie=`token=no;expires=Thu, 01 Jan 1970 00:00:01 GMT`;

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

        if(currentPage <= 0) {{
            return;
        }}

        document.cookie = `page=${{currentPage}}`;
        window.location.reload();
    }}

    function handleFilterApply(a) {{        
        if(activity.value) {{
            document.cookie = `activity=${{activity.value}}`;
        }} else {{
            eraseCookie(\"activity\");
        }}

        if(userId.value) {{
            document.cookie = `userId=${{userId.value}}`;
        }} else {{
            eraseCookie(\"userId\");
        }}

        if(status.value) {{
            document.cookie = `status=${{status.value}}`;
        }} else {{
            eraseCookie(\"status\");
        }}

        console.log(`id=${{userId.value}}; activity=${{activity.value}}; status=${{status.value}}`);
        window.location.reload();
    }}
  </script>", construct_results(data));

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
