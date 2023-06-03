(function() {

    function getURL() : string {
        const SCRIPT_NAME = "search_corpus.cgi";
        if (process.env.NODE_ENV !== "production")
        {
            return 'https://gregstoll.dyndns.org/wheeloffortune/' + SCRIPT_NAME;
        }
        return SCRIPT_NAME;
    }

    function getRowStyle(className: string): CSSStyleRule {
        const stylesheet = document.styleSheets[0];

        for(let i = 0; i < stylesheet.cssRules.length; i++) {
            const rule = stylesheet.cssRules[i] as CSSStyleRule;
            if(rule.selectorText === 'tr.' + className) {
                return rule;
            }
        }
    }
    function setMoreWordsRowStyleVisible(visible: boolean): void {
        const letterStyle = getRowStyle('more-words-row');
        letterStyle.style.display = visible ? 'table-row' : 'none';
    }

    async function fetchData(pattern: string) {
        let response = await fetch(getURL()+'?mode=Crossword&pattern='+encodeURIComponent(pattern));
        const json = await response.json();
        let word_list = document.getElementById("possible_word_list");
        if (json.error) {
            console.error(`Error from script: ${json.error}`);
            const error_text = `<span style=\"color: red;\">Error: ${json.error}</span>`;
            word_list.innerHTML = error_text;
            return;
        }
        if (json.length === 0) {
            word_list.innerHTML = "No words found";
            return;
        }
        let word_table = document.createElement("table");
        let total_frequency = 0;
        let more_words : HTMLDetailsElement | undefined = undefined;
        let word_count = 0;
        const WORD_LIMIT = 10;
        for (let result of json) {
            let row = document.createElement("tr");
            let word_td = document.createElement("td");
            word_td.appendChild(document.createTextNode(result.word));
            let freq_td = document.createElement("td");
            freq_td.classList.add("frequency");
            freq_td.appendChild(document.createTextNode(result.frequency));
            total_frequency += result.frequency;
            row.appendChild(word_td);
            row.appendChild(freq_td);

            word_count += 1;
            if(word_count > WORD_LIMIT) {
                if (more_words === undefined) {
                    more_words = document.createElement("details");
                    let summary = document.createElement("summary");
                    summary.appendChild(document.createTextNode("More words"));
                    more_words.appendChild(summary);
                    // Don't actually put a table in the details element, because then the
                    // two tables won't be the same width. Just show/hide the table when
                    // the details element is toggled.
                    more_words.addEventListener("toggle", event => {
                        const is_open = (event.target as HTMLDetailsElement).open;
                        setMoreWordsRowStyleVisible(is_open);
                    });
                    word_table.appendChild(more_words);
                }
                row.classList.add("more-words-row");
            }
            word_table.appendChild(row);
        }
        word_list.innerHTML = "";
        setMoreWordsRowStyleVisible(false);
        word_list.appendChild(word_table);
    }

    document.getElementById("search").addEventListener("click", function() {
        let pattern_element = document.getElementById("pattern") as HTMLInputElement;
        let pattern = pattern_element.value;
        pattern = pattern.trim().toLowerCase().replace(/\s+/g, '');
        // unicode ellipses
        pattern = pattern.replace(/\u2026/g, '...');
        pattern_element.value = pattern;
        pattern = pattern.replace(/\./g, "?").replace(/\*/g, "?");
        fetchData(pattern);
    });

    document.addEventListener("DOMContentLoaded", function() {
        let patternElem = document.getElementById("pattern") as HTMLInputElement;
        if (patternElem.value.trim() == "") {
            // set up default values.
            // hopefully this will avoid clearing them if the user reloads
            // (this is a problem with setting value in the HTML directly)
            patternElem.value = "??ai?";
        }
    }, false);
})();
