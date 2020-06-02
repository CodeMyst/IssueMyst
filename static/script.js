window.addEventListener("load", async () =>
{
    let error = document.querySelector(".error");
    let repoElement = document.querySelector(".repo");

    document.querySelector(".url-input input[type=button]").addEventListener("click", async () =>
    {
        let url = document.getElementById("repo-url").value;

        let match = /https:\/\/(?:www\.)?github\.com\/(.*?)\/(.*?)(?:\/|\?|$)/.exec(url);

        if (match)
        {
            repoElement.classList.remove("hidden");
            if (!error.classList.contains("hidden"))
            {
                error.classList.add("hidden");
            }

            let username = match[1];
            let repo = match[2];

            let body =
            {
                "username": username,
                "repo": repo
            };

            let res = await fetch(`${location.protocol}//${location.host}/`,
            {
                method: "POST",
                body: JSON.stringify(body),
                headers:
                {
                    "Content-Type": "application/json"
                }
            });

            console.log(await res.json());
        }
        else
        {
            error.classList.remove("hidden");
            if (!repoElement.classList.contains("hidden"))
            {
                repoElement.classList.add("hidden");
            }
        }
    });
});