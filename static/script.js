window.addEventListener("load", async () =>
{
    let error = document.querySelector(".error");
    let errorMessage = error.querySelector(".description");
    let loading = document.querySelector(".loading");
    let repoElement = document.querySelector(".repo");
    let idElement = repoElement.querySelector(".id");
    let titleElement = repoElement.querySelector(".title a");
    let labelsElement = repoElement.querySelector(".labels");

    document.querySelector(".url-input input[type=button]").addEventListener("click", async () =>
    {
        let url = document.getElementById("repo-url").value;

        let match = /https:\/\/(?:www\.)?github\.com\/(.*?)\/(.*?)(?:\/|\?|$)/.exec(url);

        if (match)
        {
            if (loading.classList.contains("hidden"))
            {
                loading.classList.remove("hidden");
            }

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

            if (res.status === 404) {
                errorMessage.textContent = "repo doesn't exist or has no open issues.";
                error.classList.remove("hidden");
                loading.classList.add("hidden");
                return;
            }

            let json;

            try
            {
                json = await res.json();
            }
            catch (e)
            {
                errorMessage.textContent = "invalid JSON returned from server, if this continues to happen contact @CodeMyst";
                error.classList.remove("hidden");
                loading.classList.add("hidden");
                return;
            }

            idElement.textContent = `[#${json.number}]`;
            titleElement.setAttribute("href", json.html_url);
            titleElement.textContent = json.title;

            labelsElement.innerHTML = "";

            for (let i = 0; i < json.labels.length; i++)
            {
                let l = document.createElement("div");
                l.classList.add("label");
                l.textContent = json.labels[i].name;
                l.style.backgroundColor = "#" + json.labels[i].color;
                if (!getColor(l.style.backgroundColor))
                {
                    l.style.color = "black";
                }

                labelsElement.appendChild(l);
            }

            loading.classList.add("hidden");

            repoElement.classList.remove("hidden");
            if (!error.classList.contains("hidden"))
            {
                error.classList.add("hidden");
            }
        }
        else
        {
            errorMessage.textContent = "invalid repo url";
            error.classList.remove("hidden");
            if (!repoElement.classList.contains("hidden"))
            {
                repoElement.classList.add("hidden");
            }
        }
    });
});

/**
 * Figures out whether to use a white or black text colour based on the background colour.
 * The colour should be in a #RRGGBB format, # is needed!
 * Returns true if the text should be white.
 */
function getColor(bgColor)
{
    let red = parseInt(bgColor.substring(1, 3), 16);
    let green = parseInt(bgColor.substring(3, 5), 16);
    let blue = parseInt(bgColor.substring(5, 7), 16);

    return (red * 0.299 + green * 0.587 + blue * 0.114) <= 186;
}