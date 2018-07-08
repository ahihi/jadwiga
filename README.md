# jadwiga

🎶 ActivityPub-speaking music/video/image publishing server

![Nodame & Jadwiga (SFX: トントコ トントコ)](https://static.foldplop.com/misc/nodame_and_jadwiga.png)

## Setup

create a key pair:

````
openssl genrsa -out private.pem 2048
openssl rsa -in private.pem -outform PEM -pubout -out public.pem
````

create [`Rocket.toml`](https://github.com/SergioBenitez/Rocket/blob/master/examples/config/Rocket.toml) if needed, e.g. to change the listening host/port 

create [`.env`](https://github.com/purpliminal/rust-dotenv):

````
JADWIGA_DATABASE_URL=database.sqlite
JADWIGA_PUBLIC_KEY=public.pem
JADWIGA_ROOT_URL=http://localhost:8000/
JADWIGA_USERNAME=admin
JADWIGA_NAME=Administrator
JADWIGA_MEDIA_DIR=media
````

make sure you have [diesel_cli](https://github.com/diesel-rs/diesel/tree/master/diesel_cli) with sqlite support:

````
$ cargo install diesel_cli --no-default-features --features sqlite
````

run migrations:

````
$ diesel migration run --database-url=database.sqlite
````

トントコ　トントコ:

````
$ cargo run
````

## Goals

- [x] Profile visible to Mastodon
- [x] Simple media store
- [ ] Posts visible to Mastodon
- [ ] Followable by Mastodon
- [ ] UI to Create post
- [ ] UI to List posts
- [ ] UI to View post
