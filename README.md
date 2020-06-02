# IssueMyst

a small website that fetches a random issue from a github repo.

## Configuration

to configure the project you just need to place `pat.txt` in the root of the project and in that put your GitHub personal access token. This is needed because the github rate limit without any authorization is only 60 requests per hour.

## Building

to build the project just run `cargo build`. you need to have setup rust nightly for this project as some libraries depend on newer features.