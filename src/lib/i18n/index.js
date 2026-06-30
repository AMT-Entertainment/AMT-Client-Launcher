import { writable, derived } from "svelte/store";
import en from "./en.json";
import de from "./de.json";
import pl from "./pl.json";

const translations = { en, de, pl };

export const locale = writable("en");

export const t = derived(locale, ($locale) => {
    return (key, params = {}) => {
        const map = translations[$locale] || translations["en"];
        const text = map?.[key] ?? translations["en"]?.[key] ?? key;
        return params ? text.replace(/\{(\w+)\}/g, (_, k) => params[k] ?? `{${k}}`) : text;
    };
});

export function initLocale(savedLang) {
    if (savedLang && ["en", "de", "pl"].includes(savedLang)) {
        locale.set(savedLang);
    }
}