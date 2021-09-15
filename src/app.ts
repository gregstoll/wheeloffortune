(function() {

    function getURL() : string {
        const SCRIPT_NAME = "search_corpus.cgi";
        if (process.env.NODE_ENV !== "production")
        {
            return 'https://gregstoll.dyndns.org/wheeloffortune/' + SCRIPT_NAME;
        }
        return SCRIPT_NAME;
    }

    async function fetchData(pattern: string, absent_letters: string) {
        let response = await fetch(getURL()+'?pattern='+encodeURIComponent(pattern)+'&absent_letters='+encodeURIComponent(absent_letters));
        const json = await response.json();
        let word_list = document.getElementById("possible_word_list") as HTMLUListElement;
        let children: HTMLElement[] = [];
        for (let result of json) {
            let result_item = document.createElement("li");
            result_item.innerHTML = result["word"] + " " + result["frequency"];
            children.push(result_item);
        }
        word_list.innerHTML = "";
        for (let child of children) {
            word_list.appendChild(child);
        }
    }

    document.getElementById("search").addEventListener("click", function() {
        let pattern = (document.getElementById("pattern") as HTMLInputElement).value;
        pattern = pattern.trim().replace(/\./g, "?");
        let absent_letters = (document.getElementById("absent_letters") as HTMLInputElement).value;
        absent_letters = absent_letters.trim();
        fetchData(pattern, absent_letters);
    });
})();