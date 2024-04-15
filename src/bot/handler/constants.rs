use chrono_tz::Tz;
use std::collections::HashMap;

pub const MAX_VALUE: i64 = 1_000_000_000_000_000_000;
pub const UNKNOWN_ERROR_MESSAGE: &str =
    "❓ Hmm, something went wrong! Sorry, I can't do that right now, please try again later!\n\n";
pub const NO_TEXT_MESSAGE: &str =
    "❓ Sorry, I can't understand that! Please reply to me in text.\n\n";
pub const TOTAL_INSTRUCTIONS_MESSAGE: &str =
"Enter the amount followed by the 3 letter abbreviation for the currency.\nE.g. 7.50 USD, 28 EUR, 320 JPY, etc.";
pub const CURRENCY_INSTRUCTIONS_MESSAGE: &str =
    "Enter the 3 letter code for the currency. E.g. USD, EUR, JPY, etc.";
pub const DEBT_EQUAL_DESCRIPTION_MESSAGE: &str =
    "Equal — Divide the total amount equally among users\n";
pub const DEBT_EXACT_DESCRIPTION_MESSAGE: &str =
    "Exact — Share the total cost by specifying exact amounts for each user\n";
pub const DEBT_RATIO_DESCRIPTION_MESSAGE: &str =
"Proportion — Split the total cost by specifying relative proportions of the total that each user owes\n";
pub const DEBT_EQUAL_INSTRUCTIONS_MESSAGE: &str =
"Enter the usernames of those sharing the cost (including the payer if sharing too) as follows: \n\n@username__1\n@username__2\n@username__3\n...\n\n";
pub const DEBT_EXACT_INSTRUCTIONS_MESSAGE: &str =
"Enter the usernames and exact amounts (without currency) as follows: \n\n@username__1 amount1\n@username__2 amount2\n@username__3 amount3\n...\n\nAny leftover amount will be taken as the payer's share.";
pub const PAY_BACK_INSTRUCTIONS_MESSAGE: &str =
"Enter the usernames and exact amounts (without currency) as follows: \n\n@username__1 amount1\n@username__2 amount2\n@username__3 amount3\n...\n\n";
pub const DEBT_RATIO_INSTRUCTIONS_MESSAGE: &str =
"Enter the usernames and proportions as follows: \n\n@username__1 portion1\n@username__2 portion2\n@username__3 portion3\n...\n\nThe portions can be any whole or decimal number.";
pub const COMMAND_HELP: &str = "/help";
pub const COMMAND_SETTINGS: &str = "/settings";
pub const COMMAND_ADD_PAYMENT: &str = "/addpayment";
pub const COMMAND_PAY_BACK: &str = "/payback";
pub const COMMAND_VIEW_PAYMENTS: &str = "/viewpayments";
pub const COMMAND_EDIT_PAYMENT: &str = "/editpayment";
pub const COMMAND_DELETE_PAYMENT: &str = "/deletepayment";
pub const COMMAND_VIEW_BALANCES: &str = "/viewbalances";

// List of all supported currencies
pub const CURRENCIES: [(&str, i32); 167] = [
    ("AED", 2),
    ("AFN", 2),
    ("ALL", 2),
    ("AMD", 2),
    ("ANG", 2),
    ("AOA", 2),
    ("ARS", 2),
    ("AUD", 2),
    ("AWG", 2),
    ("AZN", 2),
    ("BAM", 2),
    ("BBD", 2),
    ("BDT", 2),
    ("BGN", 2),
    ("BHD", 3),
    ("BIF", 0),
    ("BMD", 2),
    ("BND", 2),
    ("BOB", 2),
    ("BOV", 2),
    ("BRL", 2),
    ("BSD", 2),
    ("BTN", 2),
    ("BWP", 2),
    ("BYN", 2),
    ("BZD", 2),
    ("CAD", 2),
    ("CDF", 2),
    ("CHE", 2),
    ("CHF", 2),
    ("CHW", 2),
    ("CLF", 4),
    ("CLP", 0),
    ("CNY", 2),
    ("COP", 2),
    ("COU", 2),
    ("CRC", 2),
    ("CUP", 2),
    ("CVE", 2),
    ("CZK", 2),
    ("DJF", 0),
    ("DKK", 2),
    ("DOP", 2),
    ("DZD", 2),
    ("EGP", 2),
    ("ERN", 2),
    ("ETB", 2),
    ("EUR", 2),
    ("FJD", 2),
    ("FKP", 2),
    ("GBP", 2),
    ("GEL", 2),
    ("GHS", 2),
    ("GIP", 2),
    ("GMD", 2),
    ("GNF", 0),
    ("GTQ", 2),
    ("GYD", 2),
    ("HKD", 2),
    ("HNL", 2),
    ("HTG", 2),
    ("HUF", 2),
    ("IDR", 2),
    ("ILS", 2),
    ("INR", 2),
    ("IQD", 3),
    ("IRR", 2),
    ("ISK", 0),
    ("JMD", 2),
    ("JOD", 3),
    ("JPY", 0),
    ("KES", 2),
    ("KGS", 2),
    ("KHR", 2),
    ("KMF", 0),
    ("KPW", 2),
    ("KRW", 0),
    ("KWD", 3),
    ("KYD", 2),
    ("KZT", 2),
    ("LAK", 2),
    ("LBP", 2),
    ("LKR", 2),
    ("LRD", 2),
    ("LSL", 2),
    ("LYD", 3),
    ("MAD", 2),
    ("MDL", 2),
    ("MGA", 2),
    ("MKD", 2),
    ("MMK", 2),
    ("MNT", 2),
    ("MOP", 2),
    ("MRU", 2),
    ("MUR", 2),
    ("MVR", 2),
    ("MWK", 2),
    ("MXN", 2),
    ("MXV", 2),
    ("MYR", 2),
    ("MZN", 2),
    ("NAD", 2),
    ("NGN", 2),
    ("NIO", 2),
    ("NOK", 2),
    ("NPR", 2),
    ("NZD", 2),
    ("OMR", 3),
    ("PAB", 2),
    ("PEN", 2),
    ("PGK", 2),
    ("PHP", 2),
    ("PKR", 2),
    ("PLN", 2),
    ("PYG", 0),
    ("QAR", 2),
    ("RON", 2),
    ("RSD", 2),
    ("RUB", 2),
    ("RWF", 0),
    ("SAR", 2),
    ("SBD", 2),
    ("SCR", 2),
    ("SDG", 2),
    ("SEK", 2),
    ("SGD", 2),
    ("SHP", 2),
    ("SLE", 2),
    ("SLL", 2),
    ("SOS", 2),
    ("SRD", 2),
    ("SSP", 2),
    ("STN", 2),
    ("SVC", 2),
    ("SYP", 2),
    ("SZL", 2),
    ("THB", 2),
    ("TJS", 2),
    ("TMT", 2),
    ("TND", 3),
    ("TOP", 2),
    ("TRY", 2),
    ("TTD", 2),
    ("TWD", 2),
    ("TZS", 2),
    ("UAH", 2),
    ("UGX", 0),
    ("USD", 2),
    ("USN", 2),
    ("UYI", 0),
    ("UYU", 2),
    ("UYW", 4),
    ("UZS", 2),
    ("VED", 2),
    ("VES", 2),
    ("VND", 0),
    ("VUV", 0),
    ("WST", 2),
    ("XAF", 0),
    ("XCD", 2),
    ("XOF", 0),
    ("XPF", 0),
    ("YER", 2),
    ("ZAR", 2),
    ("ZMW", 2),
    ("ZWL", 2),
    ("NIL", 2),
];
pub const CURRENCY_DEFAULT: (&str, i32) = ("NIL", 2);

// List of all supported time zones
pub fn all_time_zones() -> HashMap<String, Tz> {
    let mut map: HashMap<String, Tz> = HashMap::new();
    map.insert("abidjan".to_string(), Tz::Africa__Abidjan);
    map.insert("accra".to_string(), Tz::Africa__Accra);
    map.insert("addis ababa".to_string(), Tz::Africa__Addis_Ababa);
    map.insert("algiers".to_string(), Tz::Africa__Algiers);
    map.insert("asmara".to_string(), Tz::Africa__Asmara);
    map.insert("asmera".to_string(), Tz::Africa__Asmera);
    map.insert("bamako".to_string(), Tz::Africa__Bamako);
    map.insert("bangui".to_string(), Tz::Africa__Bangui);
    map.insert("banjul".to_string(), Tz::Africa__Banjul);
    map.insert("bissau".to_string(), Tz::Africa__Bissau);
    map.insert("blantyre".to_string(), Tz::Africa__Blantyre);
    map.insert("brazzaville".to_string(), Tz::Africa__Brazzaville);
    map.insert("bujumbura".to_string(), Tz::Africa__Bujumbura);
    map.insert("cairo".to_string(), Tz::Africa__Cairo);
    map.insert("casablanca".to_string(), Tz::Africa__Casablanca);
    map.insert("ceuta".to_string(), Tz::Africa__Ceuta);
    map.insert("conakry".to_string(), Tz::Africa__Conakry);
    map.insert("dakar".to_string(), Tz::Africa__Dakar);
    map.insert("dar es salaam".to_string(), Tz::Africa__Dar_es_Salaam);
    map.insert("djibouti".to_string(), Tz::Africa__Djibouti);
    map.insert("douala".to_string(), Tz::Africa__Douala);
    map.insert("el aaiun".to_string(), Tz::Africa__El_Aaiun);
    map.insert("freetown".to_string(), Tz::Africa__Freetown);
    map.insert("gaborone".to_string(), Tz::Africa__Gaborone);
    map.insert("harare".to_string(), Tz::Africa__Harare);
    map.insert("johannesburg".to_string(), Tz::Africa__Johannesburg);
    map.insert("juba".to_string(), Tz::Africa__Juba);
    map.insert("kampala".to_string(), Tz::Africa__Kampala);
    map.insert("khartoum".to_string(), Tz::Africa__Khartoum);
    map.insert("kigali".to_string(), Tz::Africa__Kigali);
    map.insert("kinshasa".to_string(), Tz::Africa__Kinshasa);
    map.insert("lagos".to_string(), Tz::Africa__Lagos);
    map.insert("libreville".to_string(), Tz::Africa__Libreville);
    map.insert("lome".to_string(), Tz::Africa__Lome);
    map.insert("luanda".to_string(), Tz::Africa__Luanda);
    map.insert("lubumbashi".to_string(), Tz::Africa__Lubumbashi);
    map.insert("lusaka".to_string(), Tz::Africa__Lusaka);
    map.insert("malabo".to_string(), Tz::Africa__Malabo);
    map.insert("maputo".to_string(), Tz::Africa__Maputo);
    map.insert("maseru".to_string(), Tz::Africa__Maseru);
    map.insert("mbabane".to_string(), Tz::Africa__Mbabane);
    map.insert("mogadishu".to_string(), Tz::Africa__Mogadishu);
    map.insert("monrovia".to_string(), Tz::Africa__Monrovia);
    map.insert("nairobi".to_string(), Tz::Africa__Nairobi);
    map.insert("ndjamena".to_string(), Tz::Africa__Ndjamena);
    map.insert("niamey".to_string(), Tz::Africa__Niamey);
    map.insert("nouakchott".to_string(), Tz::Africa__Nouakchott);
    map.insert("ouagadougou".to_string(), Tz::Africa__Ouagadougou);
    map.insert("portonovo".to_string(), Tz::Africa__PortoNovo);
    map.insert("sao tome".to_string(), Tz::Africa__Sao_Tome);
    map.insert("timbuktu".to_string(), Tz::Africa__Timbuktu);
    map.insert("tripoli".to_string(), Tz::Africa__Tripoli);
    map.insert("tunis".to_string(), Tz::Africa__Tunis);
    map.insert("windhoek".to_string(), Tz::Africa__Windhoek);
    map.insert("adak".to_string(), Tz::America__Adak);
    map.insert("anchorage".to_string(), Tz::America__Anchorage);
    map.insert("anguilla".to_string(), Tz::America__Anguilla);
    map.insert("antigua".to_string(), Tz::America__Antigua);
    map.insert("araguaina".to_string(), Tz::America__Araguaina);
    map.insert("catamarca".to_string(), Tz::America__Argentina__Catamarca);
    map.insert(
        "comodrivadavia".to_string(),
        Tz::America__Argentina__ComodRivadavia,
    );
    map.insert("cordoba".to_string(), Tz::America__Argentina__Cordoba);
    map.insert("jujuy".to_string(), Tz::America__Argentina__Jujuy);
    map.insert("la rioja".to_string(), Tz::America__Argentina__La_Rioja);
    map.insert("mendoza".to_string(), Tz::America__Argentina__Mendoza);
    map.insert(
        "rio gallegos".to_string(),
        Tz::America__Argentina__Rio_Gallegos,
    );
    map.insert("salta".to_string(), Tz::America__Argentina__Salta);
    map.insert("san juan".to_string(), Tz::America__Argentina__San_Juan);
    map.insert("san luis".to_string(), Tz::America__Argentina__San_Luis);
    map.insert("tucuman".to_string(), Tz::America__Argentina__Tucuman);
    map.insert("ushuaia".to_string(), Tz::America__Argentina__Ushuaia);
    map.insert("aruba".to_string(), Tz::America__Aruba);
    map.insert("asuncion".to_string(), Tz::America__Asuncion);
    map.insert("atikokan".to_string(), Tz::America__Atikokan);
    map.insert("atka".to_string(), Tz::America__Atka);
    map.insert("bahia".to_string(), Tz::America__Bahia);
    map.insert("bahia_banderas".to_string(), Tz::America__Bahia_Banderas);
    map.insert("barbados".to_string(), Tz::America__Barbados);
    map.insert("belem".to_string(), Tz::America__Belem);
    map.insert("belize".to_string(), Tz::America__Belize);
    map.insert("blancsablon".to_string(), Tz::America__BlancSablon);
    map.insert("boa vista".to_string(), Tz::America__Boa_Vista);
    map.insert("bogota".to_string(), Tz::America__Bogota);
    map.insert("boise".to_string(), Tz::America__Boise);
    map.insert("buenos aires".to_string(), Tz::America__Buenos_Aires);
    map.insert("cambridge bay".to_string(), Tz::America__Cambridge_Bay);
    map.insert("campo grande".to_string(), Tz::America__Campo_Grande);
    map.insert("cancun".to_string(), Tz::America__Cancun);
    map.insert("caracas".to_string(), Tz::America__Caracas);
    map.insert("catamarca".to_string(), Tz::America__Catamarca);
    map.insert("cayenne".to_string(), Tz::America__Cayenne);
    map.insert("cayman".to_string(), Tz::America__Cayman);
    map.insert("chicago".to_string(), Tz::America__Chicago);
    map.insert("chihuahua".to_string(), Tz::America__Chihuahua);
    map.insert("ciudad juarez".to_string(), Tz::America__Ciudad_Juarez);
    map.insert("coral harbour".to_string(), Tz::America__Coral_Harbour);
    map.insert("cordoba".to_string(), Tz::America__Cordoba);
    map.insert("costa rica".to_string(), Tz::America__Costa_Rica);
    map.insert("creston".to_string(), Tz::America__Creston);
    map.insert("cuiaba".to_string(), Tz::America__Cuiaba);
    map.insert("curacao".to_string(), Tz::America__Curacao);
    map.insert("danmarkshavn".to_string(), Tz::America__Danmarkshavn);
    map.insert("dawson".to_string(), Tz::America__Dawson);
    map.insert("dawson creek".to_string(), Tz::America__Dawson_Creek);
    map.insert("denver".to_string(), Tz::America__Denver);
    map.insert("detroit".to_string(), Tz::America__Detroit);
    map.insert("dominica".to_string(), Tz::America__Dominica);
    map.insert("edmonton".to_string(), Tz::America__Edmonton);
    map.insert("eirunepe".to_string(), Tz::America__Eirunepe);
    map.insert("el salvador".to_string(), Tz::America__El_Salvador);
    map.insert("ensenada".to_string(), Tz::America__Ensenada);
    map.insert("fort nelson".to_string(), Tz::America__Fort_Nelson);
    map.insert("fort wayne".to_string(), Tz::America__Fort_Wayne);
    map.insert("fortaleza".to_string(), Tz::America__Fortaleza);
    map.insert("glace bay".to_string(), Tz::America__Glace_Bay);
    map.insert("godthab".to_string(), Tz::America__Godthab);
    map.insert("goose bay".to_string(), Tz::America__Goose_Bay);
    map.insert("grand turk".to_string(), Tz::America__Grand_Turk);
    map.insert("grenada".to_string(), Tz::America__Grenada);
    map.insert("guadeloupe".to_string(), Tz::America__Guadeloupe);
    map.insert("guatemala".to_string(), Tz::America__Guatemala);
    map.insert("guayaquil".to_string(), Tz::America__Guayaquil);
    map.insert("guyana".to_string(), Tz::America__Guyana);
    map.insert("halifax".to_string(), Tz::America__Halifax);
    map.insert("havana".to_string(), Tz::America__Havana);
    map.insert("hermosillo".to_string(), Tz::America__Hermosillo);
    map.insert(
        "indianapolis".to_string(),
        Tz::America__Indiana__Indianapolis,
    );
    map.insert("knox".to_string(), Tz::America__Indiana__Knox);
    map.insert("marengo".to_string(), Tz::America__Indiana__Marengo);
    map.insert("petersburg".to_string(), Tz::America__Indiana__Petersburg);
    map.insert("tell city".to_string(), Tz::America__Indiana__Tell_City);
    map.insert("vevay".to_string(), Tz::America__Indiana__Vevay);
    map.insert("vincennes".to_string(), Tz::America__Indiana__Vincennes);
    map.insert("winamac".to_string(), Tz::America__Indiana__Winamac);
    map.insert("indianapolis".to_string(), Tz::America__Indianapolis);
    map.insert("inuvik".to_string(), Tz::America__Inuvik);
    map.insert("iqaluit".to_string(), Tz::America__Iqaluit);
    map.insert("jamaica".to_string(), Tz::America__Jamaica);
    map.insert("jujuy".to_string(), Tz::America__Jujuy);
    map.insert("juneau".to_string(), Tz::America__Juneau);
    map.insert("louisville".to_string(), Tz::America__Kentucky__Louisville);
    map.insert("monticello".to_string(), Tz::America__Kentucky__Monticello);
    map.insert("knox in".to_string(), Tz::America__Knox_IN);
    map.insert("kralendijk".to_string(), Tz::America__Kralendijk);
    map.insert("la paz".to_string(), Tz::America__La_Paz);
    map.insert("lima".to_string(), Tz::America__Lima);
    map.insert("los angeles".to_string(), Tz::America__Los_Angeles);
    map.insert("louisville".to_string(), Tz::America__Louisville);
    map.insert("lower princes".to_string(), Tz::America__Lower_Princes);
    map.insert("maceio".to_string(), Tz::America__Maceio);
    map.insert("managua".to_string(), Tz::America__Managua);
    map.insert("manaus".to_string(), Tz::America__Manaus);
    map.insert("marigot".to_string(), Tz::America__Marigot);
    map.insert("martinique".to_string(), Tz::America__Martinique);
    map.insert("matamoros".to_string(), Tz::America__Matamoros);
    map.insert("mazatlan".to_string(), Tz::America__Mazatlan);
    map.insert("mendoza".to_string(), Tz::America__Mendoza);
    map.insert("menominee".to_string(), Tz::America__Menominee);
    map.insert("merida".to_string(), Tz::America__Merida);
    map.insert("metlakatla".to_string(), Tz::America__Metlakatla);
    map.insert("mexico city".to_string(), Tz::America__Mexico_City);
    map.insert("miquelon".to_string(), Tz::America__Miquelon);
    map.insert("moncton".to_string(), Tz::America__Moncton);
    map.insert("monterrey".to_string(), Tz::America__Monterrey);
    map.insert("montevideo".to_string(), Tz::America__Montevideo);
    map.insert("montreal".to_string(), Tz::America__Montreal);
    map.insert("montserrat".to_string(), Tz::America__Montserrat);
    map.insert("nassau".to_string(), Tz::America__Nassau);
    map.insert("new york".to_string(), Tz::America__New_York);
    map.insert("nipigon".to_string(), Tz::America__Nipigon);
    map.insert("nome".to_string(), Tz::America__Nome);
    map.insert("noronha".to_string(), Tz::America__Noronha);
    map.insert("beulah".to_string(), Tz::America__North_Dakota__Beulah);
    map.insert("center".to_string(), Tz::America__North_Dakota__Center);
    map.insert(
        "new salem".to_string(),
        Tz::America__North_Dakota__New_Salem,
    );
    map.insert("nuuk".to_string(), Tz::America__Nuuk);
    map.insert("ojinaga".to_string(), Tz::America__Ojinaga);
    map.insert("panama".to_string(), Tz::America__Panama);
    map.insert("pangnirtung".to_string(), Tz::America__Pangnirtung);
    map.insert("paramaribo".to_string(), Tz::America__Paramaribo);
    map.insert("phoenix".to_string(), Tz::America__Phoenix);
    map.insert("portauprince".to_string(), Tz::America__PortauPrince);
    map.insert("port of spain".to_string(), Tz::America__Port_of_Spain);
    map.insert("porto acre".to_string(), Tz::America__Porto_Acre);
    map.insert("porto velho".to_string(), Tz::America__Porto_Velho);
    map.insert("puerto rico".to_string(), Tz::America__Puerto_Rico);
    map.insert("punta arenas".to_string(), Tz::America__Punta_Arenas);
    map.insert("rainy river".to_string(), Tz::America__Rainy_River);
    map.insert("rankin inlet".to_string(), Tz::America__Rankin_Inlet);
    map.insert("recife".to_string(), Tz::America__Recife);
    map.insert("regina".to_string(), Tz::America__Regina);
    map.insert("resolute".to_string(), Tz::America__Resolute);
    map.insert("rio branco".to_string(), Tz::America__Rio_Branco);
    map.insert("rosario".to_string(), Tz::America__Rosario);
    map.insert("santa isabel".to_string(), Tz::America__Santa_Isabel);
    map.insert("santarem".to_string(), Tz::America__Santarem);
    map.insert("santiago".to_string(), Tz::America__Santiago);
    map.insert("santo domingo".to_string(), Tz::America__Santo_Domingo);
    map.insert("sao paulo".to_string(), Tz::America__Sao_Paulo);
    map.insert("scoresbysund".to_string(), Tz::America__Scoresbysund);
    map.insert("shiprock".to_string(), Tz::America__Shiprock);
    map.insert("sitka".to_string(), Tz::America__Sitka);
    map.insert("st barthelemy".to_string(), Tz::America__St_Barthelemy);
    map.insert("st johns".to_string(), Tz::America__St_Johns);
    map.insert("st kitts".to_string(), Tz::America__St_Kitts);
    map.insert("st lucia".to_string(), Tz::America__St_Lucia);
    map.insert("st thomas".to_string(), Tz::America__St_Thomas);
    map.insert("st vincent".to_string(), Tz::America__St_Vincent);
    map.insert("swift current".to_string(), Tz::America__Swift_Current);
    map.insert("tegucigalpa".to_string(), Tz::America__Tegucigalpa);
    map.insert("thule".to_string(), Tz::America__Thule);
    map.insert("thunder_bay".to_string(), Tz::America__Thunder_Bay);
    map.insert("tijuana".to_string(), Tz::America__Tijuana);
    map.insert("toronto".to_string(), Tz::America__Toronto);
    map.insert("tortola".to_string(), Tz::America__Tortola);
    map.insert("vancouver".to_string(), Tz::America__Vancouver);
    map.insert("virgin".to_string(), Tz::America__Virgin);
    map.insert("whitehorse".to_string(), Tz::America__Whitehorse);
    map.insert("winnipeg".to_string(), Tz::America__Winnipeg);
    map.insert("yakutat".to_string(), Tz::America__Yakutat);
    map.insert("yellowknife".to_string(), Tz::America__Yellowknife);
    map.insert("casey".to_string(), Tz::Antarctica__Casey);
    map.insert("davis".to_string(), Tz::Antarctica__Davis);
    map.insert("dumontdurville".to_string(), Tz::Antarctica__DumontDUrville);
    map.insert("macquarie".to_string(), Tz::Antarctica__Macquarie);
    map.insert("mawson".to_string(), Tz::Antarctica__Mawson);
    map.insert("mcmurdo".to_string(), Tz::Antarctica__McMurdo);
    map.insert("palmer".to_string(), Tz::Antarctica__Palmer);
    map.insert("rothera".to_string(), Tz::Antarctica__Rothera);
    map.insert("south pole".to_string(), Tz::Antarctica__South_Pole);
    map.insert("syowa".to_string(), Tz::Antarctica__Syowa);
    map.insert("troll".to_string(), Tz::Antarctica__Troll);
    map.insert("vostok".to_string(), Tz::Antarctica__Vostok);
    map.insert("longyearbyen".to_string(), Tz::Arctic__Longyearbyen);
    map.insert("aden".to_string(), Tz::Asia__Aden);
    map.insert("almaty".to_string(), Tz::Asia__Almaty);
    map.insert("amman".to_string(), Tz::Asia__Amman);
    map.insert("anadyr".to_string(), Tz::Asia__Anadyr);
    map.insert("aqtau".to_string(), Tz::Asia__Aqtau);
    map.insert("aqtobe".to_string(), Tz::Asia__Aqtobe);
    map.insert("ashgabat".to_string(), Tz::Asia__Ashgabat);
    map.insert("ashkhabad".to_string(), Tz::Asia__Ashkhabad);
    map.insert("atyrau".to_string(), Tz::Asia__Atyrau);
    map.insert("baghdad".to_string(), Tz::Asia__Baghdad);
    map.insert("bahrain".to_string(), Tz::Asia__Bahrain);
    map.insert("baku".to_string(), Tz::Asia__Baku);
    map.insert("bangkok".to_string(), Tz::Asia__Bangkok);
    map.insert("barnaul".to_string(), Tz::Asia__Barnaul);
    map.insert("beirut".to_string(), Tz::Asia__Beirut);
    map.insert("bishkek".to_string(), Tz::Asia__Bishkek);
    map.insert("brunei".to_string(), Tz::Asia__Brunei);
    map.insert("calcutta".to_string(), Tz::Asia__Calcutta);
    map.insert("chita".to_string(), Tz::Asia__Chita);
    map.insert("choibalsan".to_string(), Tz::Asia__Choibalsan);
    map.insert("chongqing".to_string(), Tz::Asia__Chongqing);
    map.insert("chungking".to_string(), Tz::Asia__Chungking);
    map.insert("colombo".to_string(), Tz::Asia__Colombo);
    map.insert("dacca".to_string(), Tz::Asia__Dacca);
    map.insert("damascus".to_string(), Tz::Asia__Damascus);
    map.insert("dhaka".to_string(), Tz::Asia__Dhaka);
    map.insert("dili".to_string(), Tz::Asia__Dili);
    map.insert("dubai".to_string(), Tz::Asia__Dubai);
    map.insert("dushanbe".to_string(), Tz::Asia__Dushanbe);
    map.insert("famagusta".to_string(), Tz::Asia__Famagusta);
    map.insert("gaza".to_string(), Tz::Asia__Gaza);
    map.insert("harbin".to_string(), Tz::Asia__Harbin);
    map.insert("hebron".to_string(), Tz::Asia__Hebron);
    map.insert("ho chi minh".to_string(), Tz::Asia__Ho_Chi_Minh);
    map.insert("hong kong".to_string(), Tz::Asia__Hong_Kong);
    map.insert("hovd".to_string(), Tz::Asia__Hovd);
    map.insert("irkutsk".to_string(), Tz::Asia__Irkutsk);
    map.insert("istanbul".to_string(), Tz::Asia__Istanbul);
    map.insert("jakarta".to_string(), Tz::Asia__Jakarta);
    map.insert("jayapura".to_string(), Tz::Asia__Jayapura);
    map.insert("jerusalem".to_string(), Tz::Asia__Jerusalem);
    map.insert("kabul".to_string(), Tz::Asia__Kabul);
    map.insert("kamchatka".to_string(), Tz::Asia__Kamchatka);
    map.insert("karachi".to_string(), Tz::Asia__Karachi);
    map.insert("kashgar".to_string(), Tz::Asia__Kashgar);
    map.insert("kathmandu".to_string(), Tz::Asia__Kathmandu);
    map.insert("katmandu".to_string(), Tz::Asia__Katmandu);
    map.insert("khandyga".to_string(), Tz::Asia__Khandyga);
    map.insert("kolkata".to_string(), Tz::Asia__Kolkata);
    map.insert("krasnoyarsk".to_string(), Tz::Asia__Krasnoyarsk);
    map.insert("kuala lumpur".to_string(), Tz::Asia__Kuala_Lumpur);
    map.insert("kuching".to_string(), Tz::Asia__Kuching);
    map.insert("kuwait".to_string(), Tz::Asia__Kuwait);
    map.insert("macao".to_string(), Tz::Asia__Macao);
    map.insert("macau".to_string(), Tz::Asia__Macau);
    map.insert("magadan".to_string(), Tz::Asia__Magadan);
    map.insert("makassar".to_string(), Tz::Asia__Makassar);
    map.insert("manila".to_string(), Tz::Asia__Manila);
    map.insert("muscat".to_string(), Tz::Asia__Muscat);
    map.insert("nicosia".to_string(), Tz::Asia__Nicosia);
    map.insert("novokuznetsk".to_string(), Tz::Asia__Novokuznetsk);
    map.insert("novosibirsk".to_string(), Tz::Asia__Novosibirsk);
    map.insert("omsk".to_string(), Tz::Asia__Omsk);
    map.insert("oral".to_string(), Tz::Asia__Oral);
    map.insert("phnom penh".to_string(), Tz::Asia__Phnom_Penh);
    map.insert("pontianak".to_string(), Tz::Asia__Pontianak);
    map.insert("pyongyang".to_string(), Tz::Asia__Pyongyang);
    map.insert("qatar".to_string(), Tz::Asia__Qatar);
    map.insert("qostanay".to_string(), Tz::Asia__Qostanay);
    map.insert("qyzylorda".to_string(), Tz::Asia__Qyzylorda);
    map.insert("rangoon".to_string(), Tz::Asia__Rangoon);
    map.insert("riyadh".to_string(), Tz::Asia__Riyadh);
    map.insert("saigon".to_string(), Tz::Asia__Saigon);
    map.insert("sakhalin".to_string(), Tz::Asia__Sakhalin);
    map.insert("samarkand".to_string(), Tz::Asia__Samarkand);
    map.insert("seoul".to_string(), Tz::Asia__Seoul);
    map.insert("shanghai".to_string(), Tz::Asia__Shanghai);
    map.insert("singapore".to_string(), Tz::Asia__Singapore);
    map.insert("srednekolymsk".to_string(), Tz::Asia__Srednekolymsk);
    map.insert("taipei".to_string(), Tz::Asia__Taipei);
    map.insert("tashkent".to_string(), Tz::Asia__Tashkent);
    map.insert("tbilisi".to_string(), Tz::Asia__Tbilisi);
    map.insert("tehran".to_string(), Tz::Asia__Tehran);
    map.insert("tel aviv".to_string(), Tz::Asia__Tel_Aviv);
    map.insert("thimbu".to_string(), Tz::Asia__Thimbu);
    map.insert("thimphu".to_string(), Tz::Asia__Thimphu);
    map.insert("tokyo".to_string(), Tz::Asia__Tokyo);
    map.insert("tomsk".to_string(), Tz::Asia__Tomsk);
    map.insert("ujung pandang".to_string(), Tz::Asia__Ujung_Pandang);
    map.insert("ulaanbaatar".to_string(), Tz::Asia__Ulaanbaatar);
    map.insert("ulan bator".to_string(), Tz::Asia__Ulan_Bator);
    map.insert("urumqi".to_string(), Tz::Asia__Urumqi);
    map.insert("ustnera".to_string(), Tz::Asia__UstNera);
    map.insert("vientiane".to_string(), Tz::Asia__Vientiane);
    map.insert("vladivostok".to_string(), Tz::Asia__Vladivostok);
    map.insert("yakutsk".to_string(), Tz::Asia__Yakutsk);
    map.insert("yangon".to_string(), Tz::Asia__Yangon);
    map.insert("yekaterinburg".to_string(), Tz::Asia__Yekaterinburg);
    map.insert("yerevan".to_string(), Tz::Asia__Yerevan);
    map.insert("azores".to_string(), Tz::Atlantic__Azores);
    map.insert("bermuda".to_string(), Tz::Atlantic__Bermuda);
    map.insert("canary".to_string(), Tz::Atlantic__Canary);
    map.insert("cape verde".to_string(), Tz::Atlantic__Cape_Verde);
    map.insert("faeroe".to_string(), Tz::Atlantic__Faeroe);
    map.insert("faroe".to_string(), Tz::Atlantic__Faroe);
    map.insert("jan mayen".to_string(), Tz::Atlantic__Jan_Mayen);
    map.insert("madeira".to_string(), Tz::Atlantic__Madeira);
    map.insert("reykjavik".to_string(), Tz::Atlantic__Reykjavik);
    map.insert("south georgia".to_string(), Tz::Atlantic__South_Georgia);
    map.insert("st helena".to_string(), Tz::Atlantic__St_Helena);
    map.insert("stanley".to_string(), Tz::Atlantic__Stanley);
    map.insert("act".to_string(), Tz::Australia__ACT);
    map.insert("adelaide".to_string(), Tz::Australia__Adelaide);
    map.insert("brisbane".to_string(), Tz::Australia__Brisbane);
    map.insert("broken hill".to_string(), Tz::Australia__Broken_Hill);
    map.insert("canberra".to_string(), Tz::Australia__Canberra);
    map.insert("currie".to_string(), Tz::Australia__Currie);
    map.insert("darwin".to_string(), Tz::Australia__Darwin);
    map.insert("eucla".to_string(), Tz::Australia__Eucla);
    map.insert("hobart".to_string(), Tz::Australia__Hobart);
    map.insert("lhi".to_string(), Tz::Australia__LHI);
    map.insert("lindeman".to_string(), Tz::Australia__Lindeman);
    map.insert("lord howe".to_string(), Tz::Australia__Lord_Howe);
    map.insert("melbourne".to_string(), Tz::Australia__Melbourne);
    map.insert("nsw".to_string(), Tz::Australia__NSW);
    map.insert("north".to_string(), Tz::Australia__North);
    map.insert("perth".to_string(), Tz::Australia__Perth);
    map.insert("queensland".to_string(), Tz::Australia__Queensland);
    map.insert("south".to_string(), Tz::Australia__South);
    map.insert("sydney".to_string(), Tz::Australia__Sydney);
    map.insert("tasmania".to_string(), Tz::Australia__Tasmania);
    map.insert("victoria".to_string(), Tz::Australia__Victoria);
    map.insert("west".to_string(), Tz::Australia__West);
    map.insert("yancowinna".to_string(), Tz::Australia__Yancowinna);
    map.insert("acre".to_string(), Tz::Brazil__Acre);
    map.insert("denoronha".to_string(), Tz::Brazil__DeNoronha);
    map.insert("east".to_string(), Tz::Brazil__East);
    map.insert("west".to_string(), Tz::Brazil__West);
    map.insert("cet".to_string(), Tz::CET);
    map.insert("cst6cdt".to_string(), Tz::CST6CDT);
    map.insert("atlantic".to_string(), Tz::Canada__Atlantic);
    map.insert("central".to_string(), Tz::Canada__Central);
    map.insert("eastern".to_string(), Tz::Canada__Eastern);
    map.insert("mountain".to_string(), Tz::Canada__Mountain);
    map.insert("newfoundland".to_string(), Tz::Canada__Newfoundland);
    map.insert("pacific".to_string(), Tz::Canada__Pacific);
    map.insert("saskatchewan".to_string(), Tz::Canada__Saskatchewan);
    map.insert("yukon".to_string(), Tz::Canada__Yukon);
    map.insert("continental".to_string(), Tz::Chile__Continental);
    map.insert("easterisland".to_string(), Tz::Chile__EasterIsland);
    map.insert("cuba".to_string(), Tz::Cuba);
    map.insert("eet".to_string(), Tz::EET);
    map.insert("est".to_string(), Tz::EST);
    map.insert("est5edt".to_string(), Tz::EST5EDT);
    map.insert("egypt".to_string(), Tz::Egypt);
    map.insert("eire".to_string(), Tz::Eire);
    map.insert("gmt".to_string(), Tz::Etc__GMT);
    map.insert("gmtplus0".to_string(), Tz::Etc__GMTPlus0);
    map.insert("gmtplus1".to_string(), Tz::Etc__GMTPlus1);
    map.insert("gmtplus10".to_string(), Tz::Etc__GMTPlus10);
    map.insert("gmtplus11".to_string(), Tz::Etc__GMTPlus11);
    map.insert("gmtplus12".to_string(), Tz::Etc__GMTPlus12);
    map.insert("gmtplus2".to_string(), Tz::Etc__GMTPlus2);
    map.insert("gmtplus3".to_string(), Tz::Etc__GMTPlus3);
    map.insert("gmtplus4".to_string(), Tz::Etc__GMTPlus4);
    map.insert("gmtplus5".to_string(), Tz::Etc__GMTPlus5);
    map.insert("gmtplus6".to_string(), Tz::Etc__GMTPlus6);
    map.insert("gmtplus7".to_string(), Tz::Etc__GMTPlus7);
    map.insert("gmtplus8".to_string(), Tz::Etc__GMTPlus8);
    map.insert("gmtplus9".to_string(), Tz::Etc__GMTPlus9);
    map.insert("gmtminus0".to_string(), Tz::Etc__GMTMinus0);
    map.insert("gmtminus1".to_string(), Tz::Etc__GMTMinus1);
    map.insert("gmtminus10".to_string(), Tz::Etc__GMTMinus10);
    map.insert("gmtminus11".to_string(), Tz::Etc__GMTMinus11);
    map.insert("gmtminus12".to_string(), Tz::Etc__GMTMinus12);
    map.insert("gmtminus13".to_string(), Tz::Etc__GMTMinus13);
    map.insert("gmtminus14".to_string(), Tz::Etc__GMTMinus14);
    map.insert("gmtminus2".to_string(), Tz::Etc__GMTMinus2);
    map.insert("gmtminus3".to_string(), Tz::Etc__GMTMinus3);
    map.insert("gmtminus4".to_string(), Tz::Etc__GMTMinus4);
    map.insert("gmtminus5".to_string(), Tz::Etc__GMTMinus5);
    map.insert("gmtminus6".to_string(), Tz::Etc__GMTMinus6);
    map.insert("gmtminus7".to_string(), Tz::Etc__GMTMinus7);
    map.insert("gmtminus8".to_string(), Tz::Etc__GMTMinus8);
    map.insert("gmtminus9".to_string(), Tz::Etc__GMTMinus9);
    map.insert("gmt0".to_string(), Tz::Etc__GMT0);
    map.insert("greenwich".to_string(), Tz::Etc__Greenwich);
    map.insert("uct".to_string(), Tz::Etc__UCT);
    map.insert("utc".to_string(), Tz::Etc__UTC);
    map.insert("universal".to_string(), Tz::Etc__Universal);
    map.insert("zulu".to_string(), Tz::Etc__Zulu);
    map.insert("amsterdam".to_string(), Tz::Europe__Amsterdam);
    map.insert("andorra".to_string(), Tz::Europe__Andorra);
    map.insert("astrakhan".to_string(), Tz::Europe__Astrakhan);
    map.insert("athens".to_string(), Tz::Europe__Athens);
    map.insert("belfast".to_string(), Tz::Europe__Belfast);
    map.insert("belgrade".to_string(), Tz::Europe__Belgrade);
    map.insert("berlin".to_string(), Tz::Europe__Berlin);
    map.insert("bratislava".to_string(), Tz::Europe__Bratislava);
    map.insert("brussels".to_string(), Tz::Europe__Brussels);
    map.insert("bucharest".to_string(), Tz::Europe__Bucharest);
    map.insert("budapest".to_string(), Tz::Europe__Budapest);
    map.insert("busingen".to_string(), Tz::Europe__Busingen);
    map.insert("chisinau".to_string(), Tz::Europe__Chisinau);
    map.insert("copenhagen".to_string(), Tz::Europe__Copenhagen);
    map.insert("dublin".to_string(), Tz::Europe__Dublin);
    map.insert("gibraltar".to_string(), Tz::Europe__Gibraltar);
    map.insert("guernsey".to_string(), Tz::Europe__Guernsey);
    map.insert("helsinki".to_string(), Tz::Europe__Helsinki);
    map.insert("isle of man".to_string(), Tz::Europe__Isle_of_Man);
    map.insert("istanbul".to_string(), Tz::Europe__Istanbul);
    map.insert("jersey".to_string(), Tz::Europe__Jersey);
    map.insert("kaliningrad".to_string(), Tz::Europe__Kaliningrad);
    map.insert("kiev".to_string(), Tz::Europe__Kiev);
    map.insert("kirov".to_string(), Tz::Europe__Kirov);
    map.insert("kyiv".to_string(), Tz::Europe__Kyiv);
    map.insert("lisbon".to_string(), Tz::Europe__Lisbon);
    map.insert("ljubljana".to_string(), Tz::Europe__Ljubljana);
    map.insert("london".to_string(), Tz::Europe__London);
    map.insert("luxembourg".to_string(), Tz::Europe__Luxembourg);
    map.insert("madrid".to_string(), Tz::Europe__Madrid);
    map.insert("malta".to_string(), Tz::Europe__Malta);
    map.insert("mariehamn".to_string(), Tz::Europe__Mariehamn);
    map.insert("minsk".to_string(), Tz::Europe__Minsk);
    map.insert("monaco".to_string(), Tz::Europe__Monaco);
    map.insert("moscow".to_string(), Tz::Europe__Moscow);
    map.insert("nicosia".to_string(), Tz::Europe__Nicosia);
    map.insert("oslo".to_string(), Tz::Europe__Oslo);
    map.insert("paris".to_string(), Tz::Europe__Paris);
    map.insert("podgorica".to_string(), Tz::Europe__Podgorica);
    map.insert("prague".to_string(), Tz::Europe__Prague);
    map.insert("riga".to_string(), Tz::Europe__Riga);
    map.insert("rome".to_string(), Tz::Europe__Rome);
    map.insert("samara".to_string(), Tz::Europe__Samara);
    map.insert("san marino".to_string(), Tz::Europe__San_Marino);
    map.insert("sarajevo".to_string(), Tz::Europe__Sarajevo);
    map.insert("saratov".to_string(), Tz::Europe__Saratov);
    map.insert("simferopol".to_string(), Tz::Europe__Simferopol);
    map.insert("skopje".to_string(), Tz::Europe__Skopje);
    map.insert("sofia".to_string(), Tz::Europe__Sofia);
    map.insert("stockholm".to_string(), Tz::Europe__Stockholm);
    map.insert("tallinn".to_string(), Tz::Europe__Tallinn);
    map.insert("tirane".to_string(), Tz::Europe__Tirane);
    map.insert("tiraspol".to_string(), Tz::Europe__Tiraspol);
    map.insert("ulyanovsk".to_string(), Tz::Europe__Ulyanovsk);
    map.insert("uzhgorod".to_string(), Tz::Europe__Uzhgorod);
    map.insert("vaduz".to_string(), Tz::Europe__Vaduz);
    map.insert("vatican".to_string(), Tz::Europe__Vatican);
    map.insert("vienna".to_string(), Tz::Europe__Vienna);
    map.insert("vilnius".to_string(), Tz::Europe__Vilnius);
    map.insert("volgograd".to_string(), Tz::Europe__Volgograd);
    map.insert("warsaw".to_string(), Tz::Europe__Warsaw);
    map.insert("zagreb".to_string(), Tz::Europe__Zagreb);
    map.insert("zaporozhye".to_string(), Tz::Europe__Zaporozhye);
    map.insert("zurich".to_string(), Tz::Europe__Zurich);
    map.insert("gb".to_string(), Tz::GB);
    map.insert("gbeire".to_string(), Tz::GBEire);
    map.insert("gmt".to_string(), Tz::GMT);
    map.insert("gmtplus0".to_string(), Tz::GMTPlus0);
    map.insert("gmtminus0".to_string(), Tz::GMTMinus0);
    map.insert("gmt0".to_string(), Tz::GMT0);
    map.insert("greenwich".to_string(), Tz::Greenwich);
    map.insert("hst".to_string(), Tz::HST);
    map.insert("hongkong".to_string(), Tz::Hongkong);
    map.insert("iceland".to_string(), Tz::Iceland);
    map.insert("antananarivo".to_string(), Tz::Indian__Antananarivo);
    map.insert("chagos".to_string(), Tz::Indian__Chagos);
    map.insert("christmas".to_string(), Tz::Indian__Christmas);
    map.insert("cocos".to_string(), Tz::Indian__Cocos);
    map.insert("comoro".to_string(), Tz::Indian__Comoro);
    map.insert("kerguelen".to_string(), Tz::Indian__Kerguelen);
    map.insert("mahe".to_string(), Tz::Indian__Mahe);
    map.insert("maldives".to_string(), Tz::Indian__Maldives);
    map.insert("mauritius".to_string(), Tz::Indian__Mauritius);
    map.insert("mayotte".to_string(), Tz::Indian__Mayotte);
    map.insert("reunion".to_string(), Tz::Indian__Reunion);
    map.insert("iran".to_string(), Tz::Iran);
    map.insert("israel".to_string(), Tz::Israel);
    map.insert("jamaica".to_string(), Tz::Jamaica);
    map.insert("japan".to_string(), Tz::Japan);
    map.insert("kwajalein".to_string(), Tz::Kwajalein);
    map.insert("libya".to_string(), Tz::Libya);
    map.insert("met".to_string(), Tz::MET);
    map.insert("mst".to_string(), Tz::MST);
    map.insert("mst7mdt".to_string(), Tz::MST7MDT);
    map.insert("bajanorte".to_string(), Tz::Mexico__BajaNorte);
    map.insert("bajasur".to_string(), Tz::Mexico__BajaSur);
    map.insert("general".to_string(), Tz::Mexico__General);
    map.insert("nz".to_string(), Tz::NZ);
    map.insert("nzchat".to_string(), Tz::NZCHAT);
    map.insert("navajo".to_string(), Tz::Navajo);
    map.insert("prc".to_string(), Tz::PRC);
    map.insert("pst8pdt".to_string(), Tz::PST8PDT);
    map.insert("apia".to_string(), Tz::Pacific__Apia);
    map.insert("auckland".to_string(), Tz::Pacific__Auckland);
    map.insert("bougainville".to_string(), Tz::Pacific__Bougainville);
    map.insert("chatham".to_string(), Tz::Pacific__Chatham);
    map.insert("chuuk".to_string(), Tz::Pacific__Chuuk);
    map.insert("easter".to_string(), Tz::Pacific__Easter);
    map.insert("efate".to_string(), Tz::Pacific__Efate);
    map.insert("enderbury".to_string(), Tz::Pacific__Enderbury);
    map.insert("fakaofo".to_string(), Tz::Pacific__Fakaofo);
    map.insert("fiji".to_string(), Tz::Pacific__Fiji);
    map.insert("funafuti".to_string(), Tz::Pacific__Funafuti);
    map.insert("galapagos".to_string(), Tz::Pacific__Galapagos);
    map.insert("gambier".to_string(), Tz::Pacific__Gambier);
    map.insert("guadalcanal".to_string(), Tz::Pacific__Guadalcanal);
    map.insert("guam".to_string(), Tz::Pacific__Guam);
    map.insert("honolulu".to_string(), Tz::Pacific__Honolulu);
    map.insert("johnston".to_string(), Tz::Pacific__Johnston);
    map.insert("kanton".to_string(), Tz::Pacific__Kanton);
    map.insert("kiritimati".to_string(), Tz::Pacific__Kiritimati);
    map.insert("kosrae".to_string(), Tz::Pacific__Kosrae);
    map.insert("kwajalein".to_string(), Tz::Pacific__Kwajalein);
    map.insert("majuro".to_string(), Tz::Pacific__Majuro);
    map.insert("marquesas".to_string(), Tz::Pacific__Marquesas);
    map.insert("midway".to_string(), Tz::Pacific__Midway);
    map.insert("nauru".to_string(), Tz::Pacific__Nauru);
    map.insert("niue".to_string(), Tz::Pacific__Niue);
    map.insert("norfolk".to_string(), Tz::Pacific__Norfolk);
    map.insert("noumea".to_string(), Tz::Pacific__Noumea);
    map.insert("pago pago".to_string(), Tz::Pacific__Pago_Pago);
    map.insert("palau".to_string(), Tz::Pacific__Palau);
    map.insert("pitcairn".to_string(), Tz::Pacific__Pitcairn);
    map.insert("pohnpei".to_string(), Tz::Pacific__Pohnpei);
    map.insert("ponape".to_string(), Tz::Pacific__Ponape);
    map.insert("port moresby".to_string(), Tz::Pacific__Port_Moresby);
    map.insert("rarotonga".to_string(), Tz::Pacific__Rarotonga);
    map.insert("saipan".to_string(), Tz::Pacific__Saipan);
    map.insert("samoa".to_string(), Tz::Pacific__Samoa);
    map.insert("tahiti".to_string(), Tz::Pacific__Tahiti);
    map.insert("tarawa".to_string(), Tz::Pacific__Tarawa);
    map.insert("tongatapu".to_string(), Tz::Pacific__Tongatapu);
    map.insert("truk".to_string(), Tz::Pacific__Truk);
    map.insert("wake".to_string(), Tz::Pacific__Wake);
    map.insert("wallis".to_string(), Tz::Pacific__Wallis);
    map.insert("yap".to_string(), Tz::Pacific__Yap);
    map.insert("poland".to_string(), Tz::Poland);
    map.insert("portugal".to_string(), Tz::Portugal);
    map.insert("roc".to_string(), Tz::ROC);
    map.insert("rok".to_string(), Tz::ROK);
    map.insert("singapore".to_string(), Tz::Singapore);
    map.insert("turkey".to_string(), Tz::Turkey);
    map.insert("uct".to_string(), Tz::UCT);
    map.insert("alaska".to_string(), Tz::US__Alaska);
    map.insert("aleutian".to_string(), Tz::US__Aleutian);
    map.insert("arizona".to_string(), Tz::US__Arizona);
    map.insert("central".to_string(), Tz::US__Central);
    map.insert("eastindiana".to_string(), Tz::US__EastIndiana);
    map.insert("eastern".to_string(), Tz::US__Eastern);
    map.insert("hawaii".to_string(), Tz::US__Hawaii);
    map.insert("indianastarke".to_string(), Tz::US__IndianaStarke);
    map.insert("michigan".to_string(), Tz::US__Michigan);
    map.insert("mountain".to_string(), Tz::US__Mountain);
    map.insert("pacific".to_string(), Tz::US__Pacific);
    map.insert("samoa".to_string(), Tz::US__Samoa);
    map.insert("utc".to_string(), Tz::UTC);
    map.insert("universal".to_string(), Tz::Universal);
    map.insert("wsu".to_string(), Tz::WSU);
    map.insert("wet".to_string(), Tz::WET);
    map.insert("zulu".to_string(), Tz::Zulu);

    map
}
