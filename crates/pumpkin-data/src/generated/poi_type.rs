/* This file is generated. Do not edit manually. */
use crate::tag::{RegistryKey, Taggable};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PoiType {
    Armorer = 0u8,
    Butcher = 1u8,
    Cartographer = 2u8,
    Cleric = 3u8,
    Farmer = 4u8,
    Fisherman = 5u8,
    Fletcher = 6u8,
    Leatherworker = 7u8,
    Librarian = 8u8,
    Mason = 9u8,
    Shepherd = 10u8,
    Toolsmith = 11u8,
    Weaponsmith = 12u8,
    Home = 13u8,
    Meeting = 14u8,
    Beehive = 15u8,
    BeeNest = 16u8,
    NetherPortal = 17u8,
    Lodestone = 18u8,
    TestInstance = 19u8,
    LightningRod = 20u8,
}
static POI_TYPE_BY_STATE: phf::Map<u16, PoiType> = phf::phf_map! { 24119u16 => PoiType :: Armorer , 24120u16 => PoiType :: Armorer , 24121u16 => PoiType :: Armorer , 24122u16 => PoiType :: Armorer , 24123u16 => PoiType :: Armorer , 24124u16 => PoiType :: Armorer , 24125u16 => PoiType :: Armorer , 24126u16 => PoiType :: Armorer , 24111u16 => PoiType :: Butcher , 24112u16 => PoiType :: Butcher , 24113u16 => PoiType :: Butcher , 24114u16 => PoiType :: Butcher , 24115u16 => PoiType :: Butcher , 24116u16 => PoiType :: Butcher , 24117u16 => PoiType :: Butcher , 24118u16 => PoiType :: Butcher , 24127u16 => PoiType :: Cartographer , 11186u16 => PoiType :: Cleric , 11187u16 => PoiType :: Cleric , 11188u16 => PoiType :: Cleric , 11189u16 => PoiType :: Cleric , 11190u16 => PoiType :: Cleric , 11191u16 => PoiType :: Cleric , 11192u16 => PoiType :: Cleric , 11193u16 => PoiType :: Cleric , 25100u16 => PoiType :: Farmer , 25101u16 => PoiType :: Farmer , 25102u16 => PoiType :: Farmer , 25103u16 => PoiType :: Farmer , 25104u16 => PoiType :: Farmer , 25105u16 => PoiType :: Farmer , 25106u16 => PoiType :: Farmer , 25107u16 => PoiType :: Farmer , 25108u16 => PoiType :: Farmer , 24099u16 => PoiType :: Fisherman , 24100u16 => PoiType :: Fisherman , 24101u16 => PoiType :: Fisherman , 24102u16 => PoiType :: Fisherman , 24103u16 => PoiType :: Fisherman , 24104u16 => PoiType :: Fisherman , 24105u16 => PoiType :: Fisherman , 24106u16 => PoiType :: Fisherman , 24107u16 => PoiType :: Fisherman , 24108u16 => PoiType :: Fisherman , 24109u16 => PoiType :: Fisherman , 24110u16 => PoiType :: Fisherman , 24128u16 => PoiType :: Fletcher , 11194u16 => PoiType :: Leatherworker , 11195u16 => PoiType :: Leatherworker , 11196u16 => PoiType :: Leatherworker , 11197u16 => PoiType :: Leatherworker , 11198u16 => PoiType :: Leatherworker , 11199u16 => PoiType :: Leatherworker , 11200u16 => PoiType :: Leatherworker , 11201u16 => PoiType :: Leatherworker , 24141u16 => PoiType :: Librarian , 24142u16 => PoiType :: Librarian , 24143u16 => PoiType :: Librarian , 24144u16 => PoiType :: Librarian , 24145u16 => PoiType :: Librarian , 24146u16 => PoiType :: Librarian , 24147u16 => PoiType :: Librarian , 24148u16 => PoiType :: Librarian , 24149u16 => PoiType :: Librarian , 24150u16 => PoiType :: Librarian , 24151u16 => PoiType :: Librarian , 24152u16 => PoiType :: Librarian , 24153u16 => PoiType :: Librarian , 24154u16 => PoiType :: Librarian , 24155u16 => PoiType :: Librarian , 24156u16 => PoiType :: Librarian , 24158u16 => PoiType :: Mason , 24159u16 => PoiType :: Mason , 24160u16 => PoiType :: Mason , 24161u16 => PoiType :: Mason , 24095u16 => PoiType :: Shepherd , 24096u16 => PoiType :: Shepherd , 24097u16 => PoiType :: Shepherd , 24098u16 => PoiType :: Shepherd , 24157u16 => PoiType :: Toolsmith , 24129u16 => PoiType :: Weaponsmith , 24130u16 => PoiType :: Weaponsmith , 24131u16 => PoiType :: Weaponsmith , 24132u16 => PoiType :: Weaponsmith , 24133u16 => PoiType :: Weaponsmith , 24134u16 => PoiType :: Weaponsmith , 24135u16 => PoiType :: Weaponsmith , 24136u16 => PoiType :: Weaponsmith , 24137u16 => PoiType :: Weaponsmith , 24138u16 => PoiType :: Weaponsmith , 24139u16 => PoiType :: Weaponsmith , 24140u16 => PoiType :: Weaponsmith , 2030u16 => PoiType :: Home , 2032u16 => PoiType :: Home , 2034u16 => PoiType :: Home , 2036u16 => PoiType :: Home , 2038u16 => PoiType :: Home , 2040u16 => PoiType :: Home , 2042u16 => PoiType :: Home , 2044u16 => PoiType :: Home , 2046u16 => PoiType :: Home , 2048u16 => PoiType :: Home , 2050u16 => PoiType :: Home , 2052u16 => PoiType :: Home , 2054u16 => PoiType :: Home , 2056u16 => PoiType :: Home , 2058u16 => PoiType :: Home , 2060u16 => PoiType :: Home , 2062u16 => PoiType :: Home , 2064u16 => PoiType :: Home , 2066u16 => PoiType :: Home , 2068u16 => PoiType :: Home , 2070u16 => PoiType :: Home , 2072u16 => PoiType :: Home , 2074u16 => PoiType :: Home , 2076u16 => PoiType :: Home , 2078u16 => PoiType :: Home , 2080u16 => PoiType :: Home , 2082u16 => PoiType :: Home , 2084u16 => PoiType :: Home , 2086u16 => PoiType :: Home , 2088u16 => PoiType :: Home , 2090u16 => PoiType :: Home , 2092u16 => PoiType :: Home , 2094u16 => PoiType :: Home , 2096u16 => PoiType :: Home , 2098u16 => PoiType :: Home , 2100u16 => PoiType :: Home , 2102u16 => PoiType :: Home , 2104u16 => PoiType :: Home , 2106u16 => PoiType :: Home , 2108u16 => PoiType :: Home , 2110u16 => PoiType :: Home , 2112u16 => PoiType :: Home , 2114u16 => PoiType :: Home , 2116u16 => PoiType :: Home , 2118u16 => PoiType :: Home , 2120u16 => PoiType :: Home , 2122u16 => PoiType :: Home , 2124u16 => PoiType :: Home , 2126u16 => PoiType :: Home , 2128u16 => PoiType :: Home , 2130u16 => PoiType :: Home , 2132u16 => PoiType :: Home , 2134u16 => PoiType :: Home , 2136u16 => PoiType :: Home , 2138u16 => PoiType :: Home , 2140u16 => PoiType :: Home , 2142u16 => PoiType :: Home , 2144u16 => PoiType :: Home , 2146u16 => PoiType :: Home , 2148u16 => PoiType :: Home , 2150u16 => PoiType :: Home , 2152u16 => PoiType :: Home , 2154u16 => PoiType :: Home , 2156u16 => PoiType :: Home , 2158u16 => PoiType :: Home , 2160u16 => PoiType :: Home , 2162u16 => PoiType :: Home , 2164u16 => PoiType :: Home , 2166u16 => PoiType :: Home , 2168u16 => PoiType :: Home , 2170u16 => PoiType :: Home , 2172u16 => PoiType :: Home , 2174u16 => PoiType :: Home , 2176u16 => PoiType :: Home , 2178u16 => PoiType :: Home , 2180u16 => PoiType :: Home , 2182u16 => PoiType :: Home , 2184u16 => PoiType :: Home , 2186u16 => PoiType :: Home , 2188u16 => PoiType :: Home , 2190u16 => PoiType :: Home , 2192u16 => PoiType :: Home , 2194u16 => PoiType :: Home , 2196u16 => PoiType :: Home , 2198u16 => PoiType :: Home , 2200u16 => PoiType :: Home , 2202u16 => PoiType :: Home , 2204u16 => PoiType :: Home , 2206u16 => PoiType :: Home , 2208u16 => PoiType :: Home , 2210u16 => PoiType :: Home , 2212u16 => PoiType :: Home , 2214u16 => PoiType :: Home , 2216u16 => PoiType :: Home , 2218u16 => PoiType :: Home , 2220u16 => PoiType :: Home , 2222u16 => PoiType :: Home , 2224u16 => PoiType :: Home , 2226u16 => PoiType :: Home , 2228u16 => PoiType :: Home , 2230u16 => PoiType :: Home , 2232u16 => PoiType :: Home , 2234u16 => PoiType :: Home , 2236u16 => PoiType :: Home , 2238u16 => PoiType :: Home , 2240u16 => PoiType :: Home , 2242u16 => PoiType :: Home , 2244u16 => PoiType :: Home , 2246u16 => PoiType :: Home , 2248u16 => PoiType :: Home , 2250u16 => PoiType :: Home , 2252u16 => PoiType :: Home , 2254u16 => PoiType :: Home , 2256u16 => PoiType :: Home , 2258u16 => PoiType :: Home , 2260u16 => PoiType :: Home , 2262u16 => PoiType :: Home , 2264u16 => PoiType :: Home , 2266u16 => PoiType :: Home , 2268u16 => PoiType :: Home , 2270u16 => PoiType :: Home , 2272u16 => PoiType :: Home , 2274u16 => PoiType :: Home , 2276u16 => PoiType :: Home , 2278u16 => PoiType :: Home , 2280u16 => PoiType :: Home , 2282u16 => PoiType :: Home , 2284u16 => PoiType :: Home , 24162u16 => PoiType :: Meeting , 24163u16 => PoiType :: Meeting , 24164u16 => PoiType :: Meeting , 24165u16 => PoiType :: Meeting , 24166u16 => PoiType :: Meeting , 24167u16 => PoiType :: Meeting , 24168u16 => PoiType :: Meeting , 24169u16 => PoiType :: Meeting , 24170u16 => PoiType :: Meeting , 24171u16 => PoiType :: Meeting , 24172u16 => PoiType :: Meeting , 24173u16 => PoiType :: Meeting , 24174u16 => PoiType :: Meeting , 24175u16 => PoiType :: Meeting , 24176u16 => PoiType :: Meeting , 24177u16 => PoiType :: Meeting , 24178u16 => PoiType :: Meeting , 24179u16 => PoiType :: Meeting , 24180u16 => PoiType :: Meeting , 24181u16 => PoiType :: Meeting , 24182u16 => PoiType :: Meeting , 24183u16 => PoiType :: Meeting , 24184u16 => PoiType :: Meeting , 24185u16 => PoiType :: Meeting , 24186u16 => PoiType :: Meeting , 24187u16 => PoiType :: Meeting , 24188u16 => PoiType :: Meeting , 24189u16 => PoiType :: Meeting , 24190u16 => PoiType :: Meeting , 24191u16 => PoiType :: Meeting , 24192u16 => PoiType :: Meeting , 24193u16 => PoiType :: Meeting , 25149u16 => PoiType :: Beehive , 25150u16 => PoiType :: Beehive , 25151u16 => PoiType :: Beehive , 25152u16 => PoiType :: Beehive , 25153u16 => PoiType :: Beehive , 25154u16 => PoiType :: Beehive , 25155u16 => PoiType :: Beehive , 25156u16 => PoiType :: Beehive , 25157u16 => PoiType :: Beehive , 25158u16 => PoiType :: Beehive , 25159u16 => PoiType :: Beehive , 25160u16 => PoiType :: Beehive , 25161u16 => PoiType :: Beehive , 25162u16 => PoiType :: Beehive , 25163u16 => PoiType :: Beehive , 25164u16 => PoiType :: Beehive , 25165u16 => PoiType :: Beehive , 25166u16 => PoiType :: Beehive , 25167u16 => PoiType :: Beehive , 25168u16 => PoiType :: Beehive , 25169u16 => PoiType :: Beehive , 25170u16 => PoiType :: Beehive , 25171u16 => PoiType :: Beehive , 25172u16 => PoiType :: Beehive , 25125u16 => PoiType :: BeeNest , 25126u16 => PoiType :: BeeNest , 25127u16 => PoiType :: BeeNest , 25128u16 => PoiType :: BeeNest , 25129u16 => PoiType :: BeeNest , 25130u16 => PoiType :: BeeNest , 25131u16 => PoiType :: BeeNest , 25132u16 => PoiType :: BeeNest , 25133u16 => PoiType :: BeeNest , 25134u16 => PoiType :: BeeNest , 25135u16 => PoiType :: BeeNest , 25136u16 => PoiType :: BeeNest , 25137u16 => PoiType :: BeeNest , 25138u16 => PoiType :: BeeNest , 25139u16 => PoiType :: BeeNest , 25140u16 => PoiType :: BeeNest , 25141u16 => PoiType :: BeeNest , 25142u16 => PoiType :: BeeNest , 25143u16 => PoiType :: BeeNest , 25144u16 => PoiType :: BeeNest , 25145u16 => PoiType :: BeeNest , 25146u16 => PoiType :: BeeNest , 25147u16 => PoiType :: BeeNest , 25148u16 => PoiType :: BeeNest , 8687u16 => PoiType :: NetherPortal , 8688u16 => PoiType :: NetherPortal , 25187u16 => PoiType :: Lodestone , 25099u16 => PoiType :: TestInstance , 33373u16 => PoiType :: LightningRod , 33374u16 => PoiType :: LightningRod , 33375u16 => PoiType :: LightningRod , 33376u16 => PoiType :: LightningRod , 33377u16 => PoiType :: LightningRod , 33378u16 => PoiType :: LightningRod , 33379u16 => PoiType :: LightningRod , 33380u16 => PoiType :: LightningRod , 33381u16 => PoiType :: LightningRod , 33382u16 => PoiType :: LightningRod , 33383u16 => PoiType :: LightningRod , 33384u16 => PoiType :: LightningRod , 33385u16 => PoiType :: LightningRod , 33386u16 => PoiType :: LightningRod , 33387u16 => PoiType :: LightningRod , 33388u16 => PoiType :: LightningRod , 33389u16 => PoiType :: LightningRod , 33390u16 => PoiType :: LightningRod , 33391u16 => PoiType :: LightningRod , 33392u16 => PoiType :: LightningRod , 33393u16 => PoiType :: LightningRod , 33394u16 => PoiType :: LightningRod , 33395u16 => PoiType :: LightningRod , 33396u16 => PoiType :: LightningRod , 33397u16 => PoiType :: LightningRod , 33398u16 => PoiType :: LightningRod , 33399u16 => PoiType :: LightningRod , 33400u16 => PoiType :: LightningRod , 33401u16 => PoiType :: LightningRod , 33402u16 => PoiType :: LightningRod , 33403u16 => PoiType :: LightningRod , 33404u16 => PoiType :: LightningRod , 33405u16 => PoiType :: LightningRod , 33406u16 => PoiType :: LightningRod , 33407u16 => PoiType :: LightningRod , 33408u16 => PoiType :: LightningRod , 33409u16 => PoiType :: LightningRod , 33410u16 => PoiType :: LightningRod , 33411u16 => PoiType :: LightningRod , 33412u16 => PoiType :: LightningRod , 33413u16 => PoiType :: LightningRod , 33414u16 => PoiType :: LightningRod , 33415u16 => PoiType :: LightningRod , 33416u16 => PoiType :: LightningRod , 33417u16 => PoiType :: LightningRod , 33418u16 => PoiType :: LightningRod , 33419u16 => PoiType :: LightningRod , 33420u16 => PoiType :: LightningRod , 33421u16 => PoiType :: LightningRod , 33422u16 => PoiType :: LightningRod , 33423u16 => PoiType :: LightningRod , 33424u16 => PoiType :: LightningRod , 33425u16 => PoiType :: LightningRod , 33426u16 => PoiType :: LightningRod , 33427u16 => PoiType :: LightningRod , 33428u16 => PoiType :: LightningRod , 33429u16 => PoiType :: LightningRod , 33430u16 => PoiType :: LightningRod , 33431u16 => PoiType :: LightningRod , 33432u16 => PoiType :: LightningRod , 33433u16 => PoiType :: LightningRod , 33434u16 => PoiType :: LightningRod , 33435u16 => PoiType :: LightningRod , 33436u16 => PoiType :: LightningRod , 33437u16 => PoiType :: LightningRod , 33438u16 => PoiType :: LightningRod , 33439u16 => PoiType :: LightningRod , 33440u16 => PoiType :: LightningRod , 33441u16 => PoiType :: LightningRod , 33442u16 => PoiType :: LightningRod , 33443u16 => PoiType :: LightningRod , 33444u16 => PoiType :: LightningRod , 33445u16 => PoiType :: LightningRod , 33446u16 => PoiType :: LightningRod , 33447u16 => PoiType :: LightningRod , 33448u16 => PoiType :: LightningRod , 33449u16 => PoiType :: LightningRod , 33450u16 => PoiType :: LightningRod , 33451u16 => PoiType :: LightningRod , 33452u16 => PoiType :: LightningRod , 33453u16 => PoiType :: LightningRod , 33454u16 => PoiType :: LightningRod , 33455u16 => PoiType :: LightningRod , 33456u16 => PoiType :: LightningRod , 33457u16 => PoiType :: LightningRod , 33458u16 => PoiType :: LightningRod , 33459u16 => PoiType :: LightningRod , 33460u16 => PoiType :: LightningRod , 33461u16 => PoiType :: LightningRod , 33462u16 => PoiType :: LightningRod , 33463u16 => PoiType :: LightningRod , 33464u16 => PoiType :: LightningRod , 33465u16 => PoiType :: LightningRod , 33466u16 => PoiType :: LightningRod , 33467u16 => PoiType :: LightningRod , 33468u16 => PoiType :: LightningRod , 33469u16 => PoiType :: LightningRod , 33470u16 => PoiType :: LightningRod , 33471u16 => PoiType :: LightningRod , 33472u16 => PoiType :: LightningRod , 33473u16 => PoiType :: LightningRod , 33474u16 => PoiType :: LightningRod , 33475u16 => PoiType :: LightningRod , 33476u16 => PoiType :: LightningRod , 33477u16 => PoiType :: LightningRod , 33478u16 => PoiType :: LightningRod , 33479u16 => PoiType :: LightningRod , 33480u16 => PoiType :: LightningRod , 33481u16 => PoiType :: LightningRod , 33482u16 => PoiType :: LightningRod , 33483u16 => PoiType :: LightningRod , 33484u16 => PoiType :: LightningRod , 33485u16 => PoiType :: LightningRod , 33486u16 => PoiType :: LightningRod , 33487u16 => PoiType :: LightningRod , 33488u16 => PoiType :: LightningRod , 33489u16 => PoiType :: LightningRod , 33490u16 => PoiType :: LightningRod , 33491u16 => PoiType :: LightningRod , 33492u16 => PoiType :: LightningRod , 33493u16 => PoiType :: LightningRod , 33494u16 => PoiType :: LightningRod , 33495u16 => PoiType :: LightningRod , 33496u16 => PoiType :: LightningRod , 33497u16 => PoiType :: LightningRod , 33498u16 => PoiType :: LightningRod , 33499u16 => PoiType :: LightningRod , 33500u16 => PoiType :: LightningRod , 33501u16 => PoiType :: LightningRod , 33502u16 => PoiType :: LightningRod , 33503u16 => PoiType :: LightningRod , 33504u16 => PoiType :: LightningRod , 33505u16 => PoiType :: LightningRod , 33506u16 => PoiType :: LightningRod , 33507u16 => PoiType :: LightningRod , 33508u16 => PoiType :: LightningRod , 33509u16 => PoiType :: LightningRod , 33510u16 => PoiType :: LightningRod , 33511u16 => PoiType :: LightningRod , 33512u16 => PoiType :: LightningRod , 33513u16 => PoiType :: LightningRod , 33514u16 => PoiType :: LightningRod , 33515u16 => PoiType :: LightningRod , 33516u16 => PoiType :: LightningRod , 33517u16 => PoiType :: LightningRod , 33518u16 => PoiType :: LightningRod , 33519u16 => PoiType :: LightningRod , 33520u16 => PoiType :: LightningRod , 33521u16 => PoiType :: LightningRod , 33522u16 => PoiType :: LightningRod , 33523u16 => PoiType :: LightningRod , 33524u16 => PoiType :: LightningRod , 33525u16 => PoiType :: LightningRod , 33526u16 => PoiType :: LightningRod , 33527u16 => PoiType :: LightningRod , 33528u16 => PoiType :: LightningRod , 33529u16 => PoiType :: LightningRod , 33530u16 => PoiType :: LightningRod , 33531u16 => PoiType :: LightningRod , 33532u16 => PoiType :: LightningRod , 33533u16 => PoiType :: LightningRod , 33534u16 => PoiType :: LightningRod , 33535u16 => PoiType :: LightningRod , 33536u16 => PoiType :: LightningRod , 33537u16 => PoiType :: LightningRod , 33538u16 => PoiType :: LightningRod , 33539u16 => PoiType :: LightningRod , 33540u16 => PoiType :: LightningRod , 33541u16 => PoiType :: LightningRod , 33542u16 => PoiType :: LightningRod , 33543u16 => PoiType :: LightningRod , 33544u16 => PoiType :: LightningRod , 33545u16 => PoiType :: LightningRod , 33546u16 => PoiType :: LightningRod , 33547u16 => PoiType :: LightningRod , 33548u16 => PoiType :: LightningRod , 33549u16 => PoiType :: LightningRod , 33550u16 => PoiType :: LightningRod , 33551u16 => PoiType :: LightningRod , 33552u16 => PoiType :: LightningRod , 33553u16 => PoiType :: LightningRod , 33554u16 => PoiType :: LightningRod , 33555u16 => PoiType :: LightningRod , 33556u16 => PoiType :: LightningRod , 33557u16 => PoiType :: LightningRod , 33558u16 => PoiType :: LightningRod , 33559u16 => PoiType :: LightningRod , 33560u16 => PoiType :: LightningRod , 33561u16 => PoiType :: LightningRod , 33562u16 => PoiType :: LightningRod , 33563u16 => PoiType :: LightningRod , 33564u16 => PoiType :: LightningRod , };
impl PoiType {
    pub const ALL: &'static [Self] = &[
        Self::Armorer,
        Self::Butcher,
        Self::Cartographer,
        Self::Cleric,
        Self::Farmer,
        Self::Fisherman,
        Self::Fletcher,
        Self::Leatherworker,
        Self::Librarian,
        Self::Mason,
        Self::Shepherd,
        Self::Toolsmith,
        Self::Weaponsmith,
        Self::Home,
        Self::Meeting,
        Self::Beehive,
        Self::BeeNest,
        Self::NetherPortal,
        Self::Lodestone,
        Self::TestInstance,
        Self::LightningRod,
    ];
    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Armorer => "armorer",
            Self::Butcher => "butcher",
            Self::Cartographer => "cartographer",
            Self::Cleric => "cleric",
            Self::Farmer => "farmer",
            Self::Fisherman => "fisherman",
            Self::Fletcher => "fletcher",
            Self::Leatherworker => "leatherworker",
            Self::Librarian => "librarian",
            Self::Mason => "mason",
            Self::Shepherd => "shepherd",
            Self::Toolsmith => "toolsmith",
            Self::Weaponsmith => "weaponsmith",
            Self::Home => "home",
            Self::Meeting => "meeting",
            Self::Beehive => "beehive",
            Self::BeeNest => "bee_nest",
            Self::NetherPortal => "nether_portal",
            Self::Lodestone => "lodestone",
            Self::TestInstance => "test_instance",
            Self::LightningRod => "lightning_rod",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "armorer" | "minecraft:armorer" => Some(Self::Armorer),
            "butcher" | "minecraft:butcher" => Some(Self::Butcher),
            "cartographer" | "minecraft:cartographer" => Some(Self::Cartographer),
            "cleric" | "minecraft:cleric" => Some(Self::Cleric),
            "farmer" | "minecraft:farmer" => Some(Self::Farmer),
            "fisherman" | "minecraft:fisherman" => Some(Self::Fisherman),
            "fletcher" | "minecraft:fletcher" => Some(Self::Fletcher),
            "leatherworker" | "minecraft:leatherworker" => Some(Self::Leatherworker),
            "librarian" | "minecraft:librarian" => Some(Self::Librarian),
            "mason" | "minecraft:mason" => Some(Self::Mason),
            "shepherd" | "minecraft:shepherd" => Some(Self::Shepherd),
            "toolsmith" | "minecraft:toolsmith" => Some(Self::Toolsmith),
            "weaponsmith" | "minecraft:weaponsmith" => Some(Self::Weaponsmith),
            "home" | "minecraft:home" => Some(Self::Home),
            "meeting" | "minecraft:meeting" => Some(Self::Meeting),
            "beehive" | "minecraft:beehive" => Some(Self::Beehive),
            "bee_nest" | "minecraft:bee_nest" => Some(Self::BeeNest),
            "nether_portal" | "minecraft:nether_portal" => Some(Self::NetherPortal),
            "lodestone" | "minecraft:lodestone" => Some(Self::Lodestone),
            "test_instance" | "minecraft:test_instance" => Some(Self::TestInstance),
            "lightning_rod" | "minecraft:lightning_rod" => Some(Self::LightningRod),
            _ => None,
        }
    }
    #[must_use]
    pub const fn max_tickets(self) -> i32 {
        match self {
            Self::Armorer => 1i32,
            Self::Butcher => 1i32,
            Self::Cartographer => 1i32,
            Self::Cleric => 1i32,
            Self::Farmer => 1i32,
            Self::Fisherman => 1i32,
            Self::Fletcher => 1i32,
            Self::Leatherworker => 1i32,
            Self::Librarian => 1i32,
            Self::Mason => 1i32,
            Self::Shepherd => 1i32,
            Self::Toolsmith => 1i32,
            Self::Weaponsmith => 1i32,
            Self::Home => 1i32,
            Self::Meeting => 32i32,
            Self::Beehive => 0i32,
            Self::BeeNest => 0i32,
            Self::NetherPortal => 0i32,
            Self::Lodestone => 0i32,
            Self::TestInstance => 0i32,
            Self::LightningRod => 0i32,
        }
    }
    #[must_use]
    pub const fn valid_range(self) -> i32 {
        match self {
            Self::Armorer => 1i32,
            Self::Butcher => 1i32,
            Self::Cartographer => 1i32,
            Self::Cleric => 1i32,
            Self::Farmer => 1i32,
            Self::Fisherman => 1i32,
            Self::Fletcher => 1i32,
            Self::Leatherworker => 1i32,
            Self::Librarian => 1i32,
            Self::Mason => 1i32,
            Self::Shepherd => 1i32,
            Self::Toolsmith => 1i32,
            Self::Weaponsmith => 1i32,
            Self::Home => 1i32,
            Self::Meeting => 6i32,
            Self::Beehive => 1i32,
            Self::BeeNest => 1i32,
            Self::NetherPortal => 1i32,
            Self::Lodestone => 1i32,
            Self::TestInstance => 1i32,
            Self::LightningRod => 1i32,
        }
    }
    #[must_use]
    pub fn from_state_id(state_id: u16) -> Option<Self> {
        POI_TYPE_BY_STATE.get(&state_id).copied()
    }
}
impl Taggable for PoiType {
    #[inline]
    fn tag_key() -> RegistryKey {
        RegistryKey::PointOfInterestType
    }
    #[inline]
    fn registry_key(&self) -> &str {
        self.name()
    }
    #[inline]
    fn registry_id(&self) -> u16 {
        u16::from(self.id())
    }
}
