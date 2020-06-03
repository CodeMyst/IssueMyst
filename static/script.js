let error;
let errorMessage;
let loading;
let repoElement;
let idElement;
let titleElement;
let labelsElement;

window.addEventListener("load", async () =>
{
    error = document.querySelector(".error");
    errorMessage = error.querySelector(".description");
    loading = document.querySelector(".loading");
    repoElement = document.querySelector(".repo");
    idElement = repoElement.querySelector(".id");
    titleElement = repoElement.querySelector(".title a");
    labelsElement = repoElement.querySelector(".labels");

    document.querySelector(".url-input input[type=button]").addEventListener("click", async () =>
    {
        let url = document.getElementById("repo-url").value.trim();

        let match = /(?:https:\/\/(?:www\.)?github\.com\/)?(.*?)\/(.*?)(?:\/|\?|$)/.exec(url);

        if (match)
        {
            displayLoading();

            hideError();

            hideIssue();

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

            if (res.status !== 200)
            {
                showError(await res.json());
                return;
            }

            let json;

            try
            {
                json = await res.json();
            }
            catch (e)
            {
                showError("invalid JSON returned from server, if this continues to happen contact @CodeMyst");
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

            hideLoading();

            showIssue();

            hideError();
        }
        else
        {
            showError("invalid repo url");
            hideIssue();
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

function displayLoading()
{
    if (loading.classList.contains("hidden"))
    {
        loading.classList.remove("hidden");
    }
}

function hideLoading()
{
    loading.classList.add("hidden");
}

function showError(message)
{
    errorMessage.textContent = message;
    error.classList.remove("hidden");
    hideLoading();
}

function hideError()
{
    if (!error.classList.contains("hidden"))
    {
        error.classList.add("hidden");
    }
}

function showIssue()
{
    repoElement.classList.remove("hidden");
}

function hideIssue()
{
    if (!repoElement.classList.contains("hidden"))
    {
        repoElement.classList.add("hidden");
    }
}