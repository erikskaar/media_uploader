# Media_uploader

This program lets you sync your media directory, like a NAS, with MediaCMS without having to painstakingly check for
duplicates.

## Features

- Recursively scan all sub folders for relevant files
- Automatically upload with the correct user
- Set tags based on which folders the files are located in
- Fast duplicate check
- Corrupted file check
- Uploads new files first

## Requirements

As this program does not hash files in the same way default MediaCMS does, it will require pruning all files from
MediaCMS to work correctly. To match the hashes, and make the tags work, `files/models.py` in MediaCMS does have to be
updated. The changed file is available in `mediacms_files_changed/models.py.new`, and it can be diffed against the original
found [here](https://github.com/mediacms-io/mediacms/blob/main/files/models.py) to see what changes are required.

This program requires the MediaCMS database to be exposed to wherever you host this from. This is for checking the
stored
hashes against the generated ones.

A user named `Default Uploader` must be created in MediaCMS.

## Building the project

Building can be done using `cargo build --release`. If you want
to build for a linux installation you should build using `cargo build --release --target x86_64-unknown-linux-gnu`.
Please note that if you are attempting to build it on a mounted network volume, you will have to run the build commands in
an local volume before you move them onto the network one.

To run the program, make sure you are in the same folder as the `media_uploader` binary and then run `
./media_uploader --config config.yml`. It should be able to find the `config.yml` and `.env` file if the binary is
in the default release build folder and the others are located in the project root folder, but if not, just move them
into the same folder. 

If you wish to run the program without building a release binary you should use `cargo run -- --config config.yml`.

## Limitations

The CPU-intensive encoding done by MediaCMS does limit the amount of files that can be uploaded at the same time, as the
server will respond with `504` if it gets overloaded. This results in the program having to be run several times if
the media library is large. Reducing the `number_of_threads` in `config.yml` and changing the encoding profiles in MediaCMS can reduce this somewhat, but after enough
files it will throttle even without concurrent uploads.

The MediaCMS API does not allow for setting tags directly as a request parameter. This has lead to a hacky solution of
using the description as a way to pass in tags, and when the media gets saved it splits the description on `,` to create
each tag. Depending on if one needs the description for its actual purpose this can either be irrelevant or quite bad.
For our purpose this has no effect.

For some reason, using the authorization tokens from MediaCMS does not seem to work correctly when using POST requests
from Rust. This has lead to having to store the passwords for all relevant users in the `.env` instead of tokens, which
is sub-optimal.

## Configuration and environment

### `config.yml`

The config file serves as a way to tweak the program to best suit your needs. Currently, it has two tweakable
parameters:

- `accepted_users`
    - A list of users that can upload files, else it will result to `Default Uploader`
- `number_of_threads`
    - The number of files that can be uploaded at the same time. This also includes the hashing of the local files.

### `.env`

The environment file does require a few variables to be set:

- `[USERNAME]_PASSWORD`
    - Each user in `accepted_users`, and the `Default_Uploader`, needs to have a corresponding password set.
- `DATABASE_URL`
    - Should be in the format `postgres://[username]:[password]@[URL]:5432/mediacms`, where the default username and
      password is `mediacms`.
- `ROOT_FOLDER`
    - This is the root folder where your media files are and where the program will look for media files and
      sub-folders.
      If you want to upload videos for specific users, the root folder **must** contain folders named the usernames that
      correspond to `accepted_users`. Each user should then have their files in that folder only.

## Planned work

- Update the MediaCMS API to allow for tags directly, without having to go through the hack of using the description.
- Allow for setting file formats through `config.yml`.
- Change from passwords to tokens.
- Allow for changing default uploader user through `config.yml`.
- Allow for the program to run continuously, uploading new files as they are added to the media directory.
