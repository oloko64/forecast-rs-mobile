/**
 * Interface translations. `t()` reads the reactive language, so text updates
 * as soon as the setting changes.
 */
import type { Language } from "./types";

export type Lang = "en" | "pt-BR";

const en = {
  today: "Today",
  now: "now",
  nowButton: "Now",
  chooseLocation: "Choose a location",
  toggleFavorite: "Toggle favorite",
  refresh: "Refresh",
  settings: "Settings",
  retry: "Retry",
  fallbackNotice: "{preferred} is unavailable, showing {source}.",
  loading: "Loading forecast…",
  loaderSky: "Checking the sky",
  loaderClouds: "Reading the clouds",
  loaderStations: "Asking the weather stations",
  loaderAlmost: "Almost there",
  loaderSlow: "Still working, the connection seems slow",
  loaderTap: "Tap the icon to change the weather",
  getStarted: "Search a city or use 📍 GPS to get started.",
  creditMet: "Weather data by MET Norway (CC BY 4.0)",
  creditOpenMeteo: "Weather data by Open-Meteo.com",
  temperature: "Temperature",
  rainMax: "Rain · max {value} mm/h",
  rainNone: "Rain · none",
  hour: "Hour",

  searchCity: "Search a city…",
  searchMap: "Search to move the map…",
  backToList: "Back to list",
  useMyLocation: "Use my location",
  searching: "Searching…",
  noPlaces: "No places found",
  results: "Results",
  favorites: "Favorites",
  favoritesHint: "Tap ☆ on a place to keep it here.",
  pickOnMap: "Pick on the map",
  pickOnMapHint: "Place the pin exactly where you want",
  adjustOnMap: "Adjust on map",
  findingPlace: "Finding this place…",
  movePin: "Move the map to place the pin",
  useThisLocation: "Use this location",
  mapFailed: "Couldn't load the map. Check your connection.",

  appearance: "Appearance",
  system: "System",
  light: "Light",
  dark: "Dark",
  colorTheme: "Color theme",
  customize: "Customize",
  roleAccent: "Main",
  roleTemp: "Secondary",
  roleRain: "Rain",
  paletteOcean: "Ocean",
  paletteForest: "Forest",
  paletteSunset: "Sunset",
  resetColors: "Reset {palette} to preset colors",
  hue: "Hue",
  saturation: "Saturation",
  lightness: "Lightness",
  colorLabel: "Color {value}",
  temperatureUnit: "Temperature",
  daysToForecast: "Days to forecast",
  refreshWhileOpen: "Refresh while open",
  whenComingBack: "When coming back",
  whenComingBackHint: "Refreshes on returning to the app if the forecast is older than this.",
  weatherData: "Weather data",
  fallbackHint:
    "If {preferred} fails 3 times in a row, the app switches to {other} for a while. MET Norway forecasts up to ~10 days.",
  language: "Language",

  gpsUnavailable: "GPS is not available here",
  gpsPhoneOnly: "GPS is only available on the phone app",
  permissionDenied: "Location permission denied",
  myLocation: "My location",
};

export type Key = keyof typeof en;

const ptBR: Record<Key, string> = {
  today: "Hoje",
  now: "agora",
  nowButton: "Agora",
  chooseLocation: "Escolha um local",
  toggleFavorite: "Favoritar",
  refresh: "Atualizar",
  settings: "Configurações",
  retry: "Tentar de novo",
  fallbackNotice: "{preferred} está indisponível, mostrando {source}.",
  loading: "Carregando previsão…",
  loaderSky: "Olhando o céu",
  loaderClouds: "Lendo as nuvens",
  loaderStations: "Consultando as estações",
  loaderAlmost: "Quase lá",
  loaderSlow: "Ainda carregando, a conexão parece lenta",
  loaderTap: "Toque no ícone para mudar o tempo",
  getStarted: "Busque uma cidade ou use o 📍 GPS para começar.",
  creditMet: "Dados do tempo: MET Norway (CC BY 4.0)",
  creditOpenMeteo: "Dados do tempo: Open-Meteo.com",
  temperature: "Temperatura",
  rainMax: "Chuva · máx. {value} mm/h",
  rainNone: "Chuva · nenhuma",
  hour: "Hora",

  searchCity: "Buscar cidade…",
  searchMap: "Busque para mover o mapa…",
  backToList: "Voltar à lista",
  useMyLocation: "Usar minha localização",
  searching: "Buscando…",
  noPlaces: "Nenhum local encontrado",
  results: "Resultados",
  favorites: "Favoritos",
  favoritesHint: "Toque em ☆ num local para guardá-lo aqui.",
  pickOnMap: "Escolher no mapa",
  pickOnMapHint: "Posicione o pino exatamente onde quiser",
  adjustOnMap: "Ajustar no mapa",
  findingPlace: "Identificando o local…",
  movePin: "Mova o mapa para posicionar o pino",
  useThisLocation: "Usar este local",
  mapFailed: "Não foi possível carregar o mapa. Verifique sua conexão.",

  appearance: "Aparência",
  system: "Sistema",
  light: "Claro",
  dark: "Escuro",
  colorTheme: "Tema de cores",
  customize: "Personalizar",
  roleAccent: "Principal",
  roleTemp: "Secundária",
  roleRain: "Chuva",
  paletteOcean: "Oceano",
  paletteForest: "Floresta",
  paletteSunset: "Pôr do sol",
  resetColors: "Restaurar as cores padrão de {palette}",
  hue: "Matiz",
  saturation: "Saturação",
  lightness: "Luminosidade",
  colorLabel: "Cor {value}",
  temperatureUnit: "Temperatura",
  daysToForecast: "Dias de previsão",
  refreshWhileOpen: "Atualizar com o app aberto",
  whenComingBack: "Ao voltar ao app",
  whenComingBackHint: "Atualiza ao voltar ao app se a previsão for mais antiga que isso.",
  weatherData: "Dados do tempo",
  fallbackHint:
    "Se {preferred} falhar 3 vezes seguidas, o app usa {other} por um tempo. O MET Norway prevê até ~10 dias.",
  language: "Idioma",

  gpsUnavailable: "GPS indisponível aqui",
  gpsPhoneOnly: "O GPS só está disponível no app do celular",
  permissionDenied: "Permissão de localização negada",
  myLocation: "Minha localização",
};

const DICTS: Record<Lang, Record<Key, string>> = { en, "pt-BR": ptBR };

/** Weather descriptions come from Rust in English. */
const WEATHER_PT: Record<string, string> = {
  "Clear sky": "Céu limpo",
  "Mainly clear": "Predominantemente limpo",
  "Partly cloudy": "Parcialmente nublado",
  Overcast: "Nublado",
  Fog: "Neblina",
  Drizzle: "Garoa",
  "Freezing drizzle": "Garoa congelante",
  "Slight rain": "Chuva fraca",
  "Moderate rain": "Chuva moderada",
  "Heavy rain": "Chuva forte",
  "Freezing rain": "Chuva congelante",
  "Slight snow fall": "Neve fraca",
  "Moderate snow fall": "Neve moderada",
  "Heavy snow fall": "Neve forte",
  "Snow grains": "Grãos de neve",
  "Rain showers": "Pancadas de chuva",
  "Snow showers": "Pancadas de neve",
  Thunderstorm: "Tempestade",
  "Thunderstorm with hail": "Tempestade com granizo",
  Unknown: "Desconhecido",
};

export const LANGUAGES: { id: Language; name: string }[] = [
  { id: "system", name: "" }, // shown as t("system")
  { id: "en", name: "English" },
  { id: "pt-BR", name: "Português" },
];

/** The phone's language, mapped to one we have. */
export function systemLang(): Lang {
  const primary = typeof navigator === "undefined" ? "" : (navigator.languages?.[0] ?? navigator.language ?? "");
  return primary.toLowerCase().startsWith("pt") ? "pt-BR" : "en";
}

export const resolveLang = (setting: Language): Lang => (setting === "system" ? systemLang() : setting);

export const i18n = $state<{ lang: Lang }>({ lang: systemLang() });

export function t(key: Key, params?: Record<string, string | number>): string {
  let text = DICTS[i18n.lang][key] ?? en[key];
  for (const [name, value] of Object.entries(params ?? {})) text = text.replace(`{${name}}`, String(value));
  return text;
}

export const weatherText = (description: string) =>
  i18n.lang === "pt-BR" ? (WEATHER_PT[description] ?? description) : description;
