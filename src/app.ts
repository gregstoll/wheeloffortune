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
        let word_list = document.getElementById("possible_word_list");
        let best_letters_to_guess = document.getElementById("best_letters_to_guess");
        // TODO - handle error
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
            word_table.appendChild(row);
        }

        let letter_table = document.createElement("table");
        Array.from(letter_frequency.entries()).sort((a, b) => b[1] - a[1]).forEach(([letter, frequency]) => {
            let row = document.createElement("tr");
            let letter_td = document.createElement("td");
            letter_td.appendChild(document.createTextNode(letter));
            let freq_td = document.createElement("td");
            freq_td.appendChild(document.createTextNode((frequency/total_frequency*100).toFixed(2) + "%"));
            row.appendChild(letter_td);
            row.appendChild(freq_td);
            letter_table.appendChild(row);
        });
        best_letters_to_guess.innerHTML = "";
        best_letters_to_guess.appendChild(letter_table);
        word_list.innerHTML = "";
        word_list.appendChild(word_table);
    }

    document.getElementById("search").addEventListener("click", function() {
        let pattern = (document.getElementById("pattern") as HTMLInputElement).value;
        pattern = pattern.trim().replace(/\./g, "?");
        let absent_letters = (document.getElementById("absent_letters") as HTMLInputElement).value;
        absent_letters = absent_letters.trim();
        fetchData(pattern, absent_letters);
    });
})();