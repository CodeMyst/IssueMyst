window.addEventListener("load", async () =>
{
    let error = document.querySelector(".error");
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

            let json = await res.json();

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

            repoElement.classList.remove("hidden");
            if (!error.classList.contains("hidden"))
            {
                error.classList.add("hidden");
            }
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