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
    function setMoreLettersRowStyleVisible(visible: boolean): void {
        const letterStyle = getRowStyle('more-letters-row');
        letterStyle.style.display = visible ? 'table-row' : 'none';
    }
    function setMoreWordsRowStyleVisible(visible: boolean): void {
        const letterStyle = getRowStyle('more-words-row');
        letterStyle.style.display = visible ? 'table-row' : 'none';
    }

    async function fetchData(pattern: string, absent_letters: string) {
        let response = await fetch(getURL()+'?mode=WheelOfFortune&pattern='+encodeURIComponent(pattern)+'&absent_letters='+encodeURIComponent(absent_letters));
        const json = await response.json();
        let word_list = document.getElementById("possible_word_list");
        let best_letters_to_guess = document.getElementById("best_letters_to_guess");
        if (json.error) {
            console.error(`Error from script: ${json.error}`);
            const error_text = `<span style=\"color: red;\">Error: ${json.error}</span>`;
            word_list.innerHTML = error_text;
            best_letters_to_guess.innerHTML = error_text;
            return;
        }
        if (json.length === 0) {
            word_list.innerHTML = "No words found";
            best_letters_to_guess.innerHTML = "No words found";
            return;
        }
        let word_table = document.createElement("table");
        let total_frequency = 0;
        let letter_frequency : Map<string, number> = new Map();
        for (let char of Array.from({ length: 26 }, (_, i) => String.fromCharCode('a'.charCodeAt(0) + i))) {
            letter_frequency.set(char, 0);
        }
        for(let pattern_char of pattern) {
            letter_frequency.delete(pattern_char);
        }
        for(let absent_char of absent_letters) {
            letter_frequency.delete(absent_char);
        }
        let more_words : HTMLDetailsElement | undefined = undefined;
        let word_count = 0;
        const WORD_LIMIT = 10;
        for (let result of json) {
            let row = document.createElement("tr");
            let word_td = document.createElement("td");
            word_td.appendChild(document.createTextNode(result.word));
            let letters_in_word = new Set<string>(result.word);
            for (let letter of Array.from(letters_in_word)) {
                if (letter_frequency.has(letter)) {
                    letter_frequency.set(letter, letter_frequency.get(letter) + result.frequency);
                }
            }
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

        let letter_table = document.createElement("table");
        let more_letters : HTMLDetailsElement | undefined = undefined;
        let letter_count = 0;
        const LETTER_LIMIT = 5;
        Array.from(letter_frequency.entries()).sort((a, b) => b[1] - a[1]).forEach(([letter, frequency]) => {
            let row = document.createElement("tr");
            let letter_td = document.createElement("td");
            letter_td.appendChild(document.createTextNode(letter));
            let freq_td = document.createElement("td");
            freq_td.appendChild(document.createTextNode((frequency/total_frequency*100).toFixed(2) + "%"));
            freq_td.classList.add("frequency");
            row.appendChild(letter_td);
            row.appendChild(freq_td);
            letter_count += 1;
            if(letter_count > LETTER_LIMIT) {
                if (more_letters === undefined) {
                    more_letters = document.createElement("details");
                    let summary = document.createElement("summary");
                    summary.appendChild(document.createTextNode("More letters"));
                    more_letters.appendChild(summary);
                    // Don't actually put a table in the details element, because then the
                    // two tables won't be the same width. Just show/hide the table when
                    // the details element is toggled.
                    more_letters.addEventListener("toggle", event => {
                        const isOpen = (event.target as HTMLDetailsElement).open;
                        setMoreLettersRowStyleVisible(isOpen);
                    });
                    letter_table.appendChild(more_letters);
                }
                row.classList.add("more-letters-row");
            }
            letter_table.appendChild(row);
        });
        best_letters_to_guess.innerHTML = "";
        setMoreLettersRowStyleVisible(false);
        best_letters_to_guess.appendChild(letter_table);

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

        let absent_letters_element = document.getElementById("absent_letters") as HTMLInputElement;
        let absent_letters = absent_letters_element.value;
        absent_letters = absent_letters.trim().toLowerCase().replace(/\s+/g, '');
        absent_letters_element.value = absent_letters;
        fetchData(pattern, absent_letters);
    });

    document.addEventListener("DOMContentLoaded", function() {
        let patternElem = document.getElementById("pattern") as HTMLInputElement;
        let absentElem  = document.getElementById("absent_letters") as HTMLInputElement;
        if (patternElem.value.trim() == "" && absentElem.value.trim() == "") {
            // set up default values.
            // hopefully this will avoid clearing them if the user reloads
            // (this is a problem with setting value in the HTML directly)
            patternElem.value = "??ai?";
            absentElem.value = "er";
        }
    }, false);
})();
