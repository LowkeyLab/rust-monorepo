use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

static CSS: Asset = asset!("assets/main.css");

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        Title {}
        DogView {}
    }
}

#[component]
fn Title() -> Element {
    rsx! {
        div { id: "title",
            h1 { "HotDog! 🌭" }
        }
    }
}

#[component]
fn DogView() -> Element {
    let mut img_src = use_resource(|| async move {
        reqwest::get("https://dog.ceo/api/breeds/image/random")
            .await
            .unwrap()
            .json::<DogApi>()
            .await
            .unwrap()
            .message
    });

    rsx! {
        div { id: "dogview",
            img { src: {img_src.cloned().unwrap_or_default()} },
        }
        div { id: "buttons",
            button { onclick: move |_| img_src.restart(), id: "skip", "skip" }
            button { id: "save", onclick: move |_| async move {
                let image = img_src.cloned().unwrap();
                img_src.restart();
                _  = save_dog(image).await;
            } ,"save!" }
        }
    }
}

#[derive(serde::Deserialize)]
struct DogApi {
    message: String,
}

// Expose a `save_dog` endpoint on our server that takes an "image" parameter
#[server]
async fn save_dog(image: String) -> Result<(), ServerFnError> {
    DB.with(|f| f.execute("INSERT INTO dogs (url) VALUES (?1)", &[&image]))?;
    Ok(())
}

// The database is only available to server code
#[cfg(feature = "server")]
thread_local! {
    pub static DB: rusqlite::Connection = {
        // Open the database from the persisted "hotdog.db" file
        let conn = rusqlite::Connection::open("hotdog.db").expect("Failed to open database");

        // Create the "dogs" table if it doesn't already exist
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS dogs (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL
            );",
        ).unwrap();

        // Return the connection
        conn
    };
}
