#![allow(non_upper_case_globals)]

use anyhow::Result;
use livesplit_core::TimeSpan;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::ops::Index;
use std::sync::Arc;
use std::time::Instant;
use time::Duration;

use super::{AutoSplitter, SNESSummary};

lazy_static! {
    static ref roomIDEnum: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert("landingSite",                    0x91F8 );
        m.insert("crateriaPowerBombRoom",          0x93AA );
        m.insert("westOcean",                      0x93FE );
        m.insert("elevatorToMaridia",              0x94CC );
        m.insert("crateriaMoat",                   0x95FF );
        m.insert("elevatorToCaterpillar",          0x962A );
        m.insert("gauntletETankRoom",              0x965B );
        m.insert("climb",                          0x96BA );
        m.insert("pitRoom",                        0x975C );
        m.insert("elevatorToMorphBall",            0x97B5 );
        m.insert("bombTorizo",                     0x9804 );
        m.insert("terminator",                     0x990D );
        m.insert("elevatorToGreenBrinstar",        0x9938 );
        m.insert("greenPirateShaft",               0x99BD );
        m.insert("crateriaSupersRoom",             0x99F9 );
        m.insert("theFinalMissile",                0x9A90 );
        m.insert("greenBrinstarMainShaft",         0x9AD9 );
        m.insert("sporeSpawnSuper",                0x9B5B );
        m.insert("earlySupers",                    0x9BC8 );
        m.insert("brinstarReserveRoom",            0x9C07 );
        m.insert("bigPink",                        0x9D19 );
        m.insert("sporeSpawnKeyhunter",            0x9D9C );
        m.insert("sporeSpawn",                     0x9DC7 );
        m.insert("pinkBrinstarPowerBombRoom",      0x9E11 );
        m.insert("greenHills",                     0x9E52 );
        m.insert("noobBridge",                     0x9FBA );
        m.insert("morphBall",                      0x9E9F );
        m.insert("blueBrinstarETankRoom",          0x9F64 );
        m.insert("etecoonETankRoom",               0xA011 );
        m.insert("etecoonSuperRoom",               0xA051 );
        m.insert("waterway",                       0xA0D2 );
        m.insert("alphaMissileRoom",               0xA107 );
        m.insert("hopperETankRoom",                0xA15B );
        m.insert("billyMays",                      0xA1D8 );
        m.insert("redTower",                       0xA253 );
        m.insert("xRay",                           0xA2CE );
        m.insert("caterpillar",                    0xA322 );
        m.insert("betaPowerBombRoom",              0xA37C );
        m.insert("alphaPowerBombsRoom",            0xA3AE );
        m.insert("bat",                            0xA3DD );
        m.insert("spazer",                         0xA447 );
        m.insert("warehouseETankRoom",             0xA4B1 );
        m.insert("warehouseZeela",                 0xA471 );
        m.insert("warehouseKiHunters",             0xA4DA );
        m.insert("kraidEyeDoor",                   0xA56B );
        m.insert("kraid",                          0xA59F );
        m.insert("statuesHallway",                 0xA5ED );
        m.insert("statues",                        0xA66A );
        m.insert("warehouseEntrance",              0xA6A1 );
        m.insert("varia",                          0xA6E2 );
        m.insert("cathedral",                      0xA788 );
        m.insert("businessCenter",                 0xA7DE );
        m.insert("iceBeam",                        0xA890 );
        m.insert("crumbleShaft",                   0xA8F8 );
        m.insert("crocomireSpeedway",              0xA923 );
        m.insert("crocomire",                      0xA98D );
        m.insert("hiJump",                         0xA9E5 );
        m.insert("crocomireEscape",                0xAA0E );
        m.insert("hiJumpShaft",                    0xAA41 );
        m.insert("postCrocomirePowerBombRoom",     0xAADE );
        m.insert("cosineRoom",                     0xAB3B );
        m.insert("preGrapple",                     0xAB8F );
        m.insert("grapple",                        0xAC2B );
        m.insert("norfairReserveRoom",             0xAC5A );
        m.insert("greenBubblesRoom",               0xAC83 );
        m.insert("bubbleMountain",                 0xACB3 );
        m.insert("speedBoostHall",                 0xACF0 );
        m.insert("speedBooster",                   0xAD1B );
        m.insert("singleChamber",                  0xAD5E ); // Exit room from Lower Norfair, also on the path to Wave
        m.insert("doubleChamber",                  0xADAD );
        m.insert("waveBeam",                       0xADDE );
        m.insert("volcano",                        0xAE32 );
        m.insert("kronicBoost",                    0xAE74 );
        m.insert("magdolliteTunnel",               0xAEB4 );
        m.insert("lowerNorfairElevator",           0xAF3F );
        m.insert("risingTide",                     0xAFA3 );
        m.insert("spikyAcidSnakes",                0xAFFB );
        m.insert("acidStatue",                     0xB1E5 );
        m.insert("mainHall",                       0xB236 ); // First room in Lower Norfair
        m.insert("goldenTorizo",                   0xB283 );
        m.insert("ridley",                         0xB32E );
        m.insert("lowerNorfairFarming",            0xB37A );
        m.insert("mickeyMouse",                    0xB40A );
        m.insert("pillars",                        0xB457 );
        m.insert("writg",                          0xB4AD );
        m.insert("amphitheatre",                   0xB4E5 );
        m.insert("lowerNorfairSpringMaze",         0xB510 );
        m.insert("lowerNorfairEscapePowerBombRoom",0xB55A );
        m.insert("redKiShaft",                     0xB585 );
        m.insert("wasteland",                      0xB5D5 );
        m.insert("metalPirates",                   0xB62B );
        m.insert("threeMusketeers",                0xB656 );
        m.insert("ridleyETankRoom",                0xB698 );
        m.insert("screwAttack",                    0xB6C1 );
        m.insert("lowerNorfairFireflea",           0xB6EE );
        m.insert("bowling",                        0xC98E );
        m.insert("wreckedShipEntrance",            0xCA08 );
        m.insert("attic",                          0xCA52 );
        m.insert("atticWorkerRobotRoom",           0xCAAE );
        m.insert("wreckedShipMainShaft",           0xCAF6 );
        m.insert("wreckedShipETankRoom",           0xCC27 );
        m.insert("basement",                       0xCC6F ); // Basement of Wrecked Ship
        m.insert("phantoon",                       0xCD13 );
        m.insert("wreckedShipLeftSuperRoom",       0xCDA8 );
        m.insert("wreckedShipRightSuperRoom",      0xCDF1 );
        m.insert("gravity",                        0xCE40 );
        m.insert("glassTunnel",                    0xCEFB );
        m.insert("mainStreet",                     0xCFC9 );
        m.insert("mamaTurtle",                     0xD055 );
        m.insert("wateringHole",                   0xD13B );
        m.insert("beach",                          0xD1DD );
        m.insert("plasmaBeam",                     0xD2AA );
        m.insert("maridiaElevator",                0xD30B );
        m.insert("plasmaSpark",                    0xD340 );
        m.insert("toiletBowl",                     0xD408 );
        m.insert("oasis",                          0xD48E );
        m.insert("leftSandPit",                    0xD4EF );
        m.insert("rightSandPit",                   0xD51E );
        m.insert("aqueduct",                       0xD5A7 );
        m.insert("butterflyRoom",                  0xD5EC );
        m.insert("botwoonHallway",                 0xD617 );
        m.insert("springBall",                     0xD6D0 );
        m.insert("precious",                       0xD78F );
        m.insert("botwoonETankRoom",               0xD7E4 );
        m.insert("botwoon",                        0xD95E );
        m.insert("spaceJump",                      0xD9AA );
        m.insert("westCactusAlley",                0xD9FE );
        m.insert("draygon",                        0xDA60 );
        m.insert("tourianElevator",                0xDAAE );
        m.insert("metroidOne",                     0xDAE1 );
        m.insert("metroidTwo",                     0xDB31 );
        m.insert("metroidThree",                   0xDB7D );
        m.insert("metroidFour",                    0xDBCD );
        m.insert("dustTorizo",                     0xDC65 );
        m.insert("tourianHopper",                  0xDC19 );
        m.insert("tourianEyeDoor",                 0xDDC4 );
        m.insert("bigBoy",                         0xDCB1 );
        m.insert("motherBrain",                    0xDD58 );
        m.insert("rinkaShaft",                     0xDDF3 );
        m.insert("tourianEscape4",                 0xDEDE );
        m.insert("ceresElevator",                  0xDF45 );
        m.insert("flatRoom",                       0xE06B ); // Placeholder name for the flat room in Ceres Station
        m.insert("ceresRidley",                    0xE0B5 );
        m
    };
    static ref mapInUseEnum: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert( "crateria",   0x0 );
        m.insert( "brinstar",   0x1 );
        m.insert( "norfair",    0x2 );
        m.insert( "wreckedShip",0x3 );
        m.insert( "maridia",    0x4 );
        m.insert( "tourian",    0x5 );
        m.insert( "ceres",      0x6 );
        m
    };

    static ref gameStateEnum: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert( "normalGameplay",         0x8  );
        m.insert( "doorTransition",         0xB  );
        m.insert( "startOfCeresCutscene",   0x20 );
        m.insert( "preEndCutscene",         0x26 ); // briefly at this value during the black screen transition after the ship fades out
        m.insert( "endCutscene",            0x27 );
        m
    };

    static ref unlockFlagEnum: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        // First item byte
        m.insert( "variaSuit",      0x1 );
        m.insert( "springBall",     0x2 );
        m.insert( "morphBall",      0x4 );
        m.insert( "screwAttack",    0x8 );
        m.insert( "gravSuit",       0x20);
        // Second item byte
        m.insert( "hiJump",         0x1 );
        m.insert( "spaceJump",      0x2 );
        m.insert( "bomb",           0x10);
        m.insert( "speedBooster",   0x20);
        m.insert( "grapple",        0x40);
        m.insert( "xray",           0x80);
        // Beams
        m.insert( "wave",           0x1 );
        m.insert( "ice",            0x2 );
        m.insert( "spazer",         0x4 );
        m.insert( "plasma",         0x8 );
        // Charge
        m.insert( "chargeBeam",     0x10);
        m
    };

    static ref motherBrainMaxHPEnum: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert( "phase1", 0xBB8  );    // 3000
        m.insert( "phase2", 0x4650 );   // 18000
        m.insert( "phase3", 0x8CA0 );   // 36000
        m
    };

    static ref eventFlagEnum: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert( "zebesAblaze",    0x40 );
        m.insert( "tubeBroken",     0x8  );
        m
    };

    static ref bossFlagEnum: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        // Crateria
        m.insert( "bombTorizo",     0x4 );
        // Brinstar
        m.insert( "sporeSpawn",     0x2 );
        m.insert( "kraid",          0x1 );
        // Norfair
        m.insert( "ridley",         0x1 );
        m.insert( "crocomire",      0x2 );
        m.insert( "goldenTorizo",   0x4 );
        // Wrecked Ship
        m.insert( "phantoon",       0x1 );
        // Maridia
        m.insert( "draygon",        0x1 );
        m.insert( "botwoon",        0x2 );
        // Tourian
        m.insert( "motherBrain",    0x2 );
        // Ceres
        m.insert( "ceresRidley",    0x1 );
        m
    };
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    data: HashMap<std::sync::Arc<str>, (bool, Option<String>)>,
    #[serde(skip)]
    modified_after_creation: bool,
}

impl Settings {
    pub fn new() -> Self {
        let mut settings = Settings {
            data: HashMap::new(),
            modified_after_creation: false,
        };
        // Split on Missiles, Super Missiles, and Power Bombs
        settings.insert("ammoPickups", true);
        // Split on the first Missile pickup
        settings.insert_with_parent("firstMissile", false, "ammoPickups");
        // Split on each Missile upgrade
        settings.insert_with_parent("allMissiles", false, "ammoPickups");
        // Split on specific Missile Pack locations
        settings.insert_with_parent("specificMissiles", false, "ammoPickups");
        // Split on Crateria Missile Pack locations
        settings.insert_with_parent("crateriaMissiles", false, "specificMissiles");
        // Split on picking up the Missile Pack located at the bottom left of the West Ocean
        settings.insert_with_parent("oceanBottomMissiles", false, "crateriaMissiles");
        // Split on picking up the Missile Pack located in the ceiling tile in West Ocean
        settings.insert_with_parent("oceanTopMissiles", false, "crateriaMissiles");
        // Split on picking up the Missile Pack located in the Morphball maze section of West Ocean
        settings.insert_with_parent("oceanMiddleMissiles", false, "crateriaMissiles");
        // Split on picking up the Missile Pack in The Moat, also known as The Lake
        settings.insert_with_parent("moatMissiles", false, "crateriaMissiles");
        // Split on picking up the Missile Pack in the Pit Room
        settings.insert_with_parent("oldTourianMissiles", false, "crateriaMissiles");
        // Split on picking up the right side Missile Pack at the end of Gauntlet(Green Pirates Shaft)
        settings.insert_with_parent("gauntletRightMissiles", false, "crateriaMissiles");
        // Split on picking up the left side Missile Pack at the end of Gauntlet(Green Pirates Shaft)
        settings.insert_with_parent("gauntletLeftMissiles", false, "crateriaMissiles");
        // Split on picking up the Missile Pack located in The Final Missile
        settings.insert_with_parent("dentalPlan", false, "crateriaMissiles");
        // Split on Brinstar Missile Pack locations
        settings.insert_with_parent("brinstarMissiles", false, "specificMissiles");
        // Split on picking up the Missile Pack located below the crumble bridge in the Early Supers Room
        settings.insert_with_parent("earlySuperBridgeMissiles", false, "brinstarMissiles");
        // Split on picking up the first Missile Pack behind the Brinstar Reserve Tank
        settings.insert_with_parent("greenBrinstarReserveMissiles", false, "brinstarMissiles");
        // Split on picking up the second Missile Pack behind the Brinstar Reserve Tank Room
        settings.insert_with_parent(
            "greenBrinstarExtraReserveMissiles",
            false,
            "brinstarMissiles",
        );
        // Split on picking up the Missile Pack located left of center in Big Pink
        settings.insert_with_parent("bigPinkTopMissiles", false, "brinstarMissiles");
        // Split on picking up the Missile Pack located at the bottom left of Big Pink
        settings.insert_with_parent("chargeMissiles", false, "brinstarMissiles");
        // Split on picking up the Missile Pack in Green Hill Zone
        settings.insert_with_parent("greenHillsMissiles", false, "brinstarMissiles");
        // Split on picking up the Missile Pack in the Blue Brinstar Energy Tank Room
        settings.insert_with_parent("blueBrinstarETankMissiles", false, "brinstarMissiles");
        // Split on picking up the first Missile Pack of the game(First Missile Room)
        settings.insert_with_parent("alphaMissiles", false, "brinstarMissiles");
        // Split on picking up the Missile Pack located on the pedestal in Billy Mays' Room
        settings.insert_with_parent("billyMaysMissiles", false, "brinstarMissiles");
        // Split on picking up the Missile Pack located in the floor of Billy Mays' Room
        settings.insert_with_parent("butWaitTheresMoreMissiles", false, "brinstarMissiles");
        // Split on picking up the Missile Pack in the Alpha Power Bombs Room
        settings.insert_with_parent("redBrinstarMissiles", false, "brinstarMissiles");
        // Split on picking up the Missile Pack in the Warehouse Kihunter Room
        settings.insert_with_parent("warehouseMissiles", false, "brinstarMissiles");
        // Split on Norfair Missile Pack locations
        settings.insert_with_parent("norfairMissiles", false, "specificMissiles");
        // Split on picking up the Missile Pack in Cathedral
        settings.insert_with_parent("cathedralMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in Crumble Shaft
        settings.insert_with_parent("crumbleShaftMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in Crocomire Escape
        settings.insert_with_parent("crocomireEscapeMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in the Hi Jump Energy Tank Room
        settings.insert_with_parent("hiJumpMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in the Post Crocomire Missile Room, also known as Cosine Room
        settings.insert_with_parent("postCrocomireMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in the Post Crocomire Jump Room
        settings.insert_with_parent("grappleMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in the Norfair Reserve Tank Room
        settings.insert_with_parent("norfairReserveMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in the Green Bubbles Missile Room
        settings.insert_with_parent("greenBubblesMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in Bubble Mountain
        settings.insert_with_parent("bubbleMountainMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in Speed Booster Hall
        settings.insert_with_parent("speedBoostMissiles", false, "norfairMissiles");
        // Split on picking up the Wave Missile Pack in Double Chamber
        settings.insert_with_parent("waveMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in the Golden Torizo's Room
        settings.insert_with_parent("goldTorizoMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in the Mickey Mouse Room
        settings.insert_with_parent("mickeyMouseMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in the Lower Norfair Springball Maze Room
        settings.insert_with_parent("lowerNorfairSpringMazeMissiles", false, "norfairMissiles");
        // Split on picking up the Missile Pack in the The Musketeers' Room
        settings.insert_with_parent("threeMusketeersMissiles", false, "norfairMissiles");
        // Split on Wrecked Ship Missile Pack locations
        settings.insert_with_parent("wreckedShipMissiles", false, "specificMissiles");
        // Split on picking up the Missile Pack in Wrecked Ship Main Shaft
        settings.insert_with_parent("wreckedShipMainShaftMissiles", false, "wreckedShipMissiles");
        // Split on picking up the Missile Pack in Bowling Alley
        settings.insert_with_parent("bowlingMissiles", false, "wreckedShipMissiles");
        // Split on picking up the Missile Pack in the Wrecked Ship East Missile Room
        settings.insert_with_parent("atticMissiles", false, "wreckedShipMissiles");
        // Split on Maridia Missile Pack locations
        settings.insert_with_parent("maridiaMissiles", false, "specificMissiles");
        // Split on picking up the Missile Pack in Main Street
        settings.insert_with_parent("mainStreetMissiles", false, "maridiaMissiles");
        // Split on picking up the Missile Pack in the Mama Turtle Room
        settings.insert_with_parent("mamaTurtleMissiles", false, "maridiaMissiles");
        // Split on picking up the Missile Pack in Watering Hole
        settings.insert_with_parent("wateringHoleMissiles", false, "maridiaMissiles");
        // Split on picking up the Missile Pack in the Pseudo Plasma Spark Room
        settings.insert_with_parent("beachMissiles", false, "maridiaMissiles");
        // Split on picking up the Missile Pack in West Sand Hole
        settings.insert_with_parent("leftSandPitMissiles", false, "maridiaMissiles");
        // Split on picking up the Missile Pack in East Sand Hole
        settings.insert_with_parent("rightSandPitMissiles", false, "maridiaMissiles");
        // Split on picking up the Missile Pack in Aqueduct
        settings.insert_with_parent("aqueductMissiles", false, "maridiaMissiles");
        // Split on picking up the Missile Pack in The Precious Room
        settings.insert_with_parent("preDraygonMissiles", false, "maridiaMissiles");
        // Split on the first Super Missile pickup
        settings.insert_with_parent("firstSuper", false, "ammoPickups");
        // Split on each Super Missile upgrade
        settings.insert_with_parent("allSupers", false, "ammoPickups");
        // Split on specific Super Missile Pack locations
        settings.insert_with_parent("specificSupers", false, "ammoPickups");
        // Split on picking up the Super Missile Pack in the Crateria Super Room
        settings.insert_with_parent("climbSupers", false, "specificSupers");
        // Split on picking up the Super Missile Pack in the Spore Spawn Super Room (NOTE: SSTRA splits when the dialogue box disappears, not on touch. Use Spore Spawn RTA Finish for SSTRA runs.)
        settings.insert_with_parent("sporeSpawnSupers", false, "specificSupers");
        // Split on picking up the Super Missile Pack in the Early Supers Room
        settings.insert_with_parent("earlySupers", false, "specificSupers");
        // Split on picking up the Super Missile Pack in the Etecoon Super Room
        settings.insert_with_parent("etecoonSupers", false, "specificSupers");
        // Split on picking up the Super Missile Pack in the Golden Torizo's Room
        settings.insert_with_parent("goldTorizoSupers", false, "specificSupers");
        // Split on picking up the Super Missile Pack in the Wrecked Ship West Super Room
        settings.insert_with_parent("wreckedShipLeftSupers", false, "specificSupers");
        // Split on picking up the Super Missile Pack in the Wrecked Ship East Super Room
        settings.insert_with_parent("wreckedShipRightSupers", false, "specificSupers");
        // Split on picking up the Super Missile Pack in Main Street
        settings.insert_with_parent("crabSupers", false, "specificSupers");
        // Split on picking up the Super Missile Pack in Watering Hole
        settings.insert_with_parent("wateringHoleSupers", false, "specificSupers");
        // Split on picking up the Super Missile Pack in Aqueduct
        settings.insert_with_parent("aqueductSupers", false, "specificSupers");
        // Split on the first Power Bomb pickup
        settings.insert_with_parent("firstPowerBomb", true, "ammoPickups");
        // Split on each Power Bomb upgrade
        settings.insert_with_parent("allPowerBombs", false, "ammoPickups");
        // Split on specific Power Bomb Pack locations
        settings.insert_with_parent("specificBombs", false, "ammoPickups");
        // Split on picking up the Power Bomb Pack in the Crateria Power Bomb Room
        settings.insert_with_parent("landingSiteBombs", false, "specificBombs");
        // Split on picking up the Power Bomb Pack in the Etecoon Room section of Green Brinstar Main Shaft
        settings.insert_with_parent("etecoonBombs", false, "specificBombs");
        // Split on picking up the Power Bomb Pack in the Pink Brinstar Power Bomb Room
        settings.insert_with_parent("pinkBrinstarBombs", false, "specificBombs");
        // Split on picking up the Power Bomb Pack in the Morph Ball Room
        settings.insert_with_parent("blueBrinstarBombs", false, "specificBombs");
        // Split on picking up the Power Bomb Pack in the Alpha Power Bomb Room
        settings.insert_with_parent("alphaBombs", false, "specificBombs");
        // Split on picking up the Power Bomb Pack in the Beta Power Bomb Room
        settings.insert_with_parent("betaBombs", false, "specificBombs");
        // Split on picking up the Power Bomb Pack in the Post Crocomire Power Bomb Room
        settings.insert_with_parent("crocomireBombs", false, "specificBombs");
        // Split on picking up the Power Bomb Pack in the Lower Norfair Escape Power Bomb Room
        settings.insert_with_parent("lowerNorfairEscapeBombs", false, "specificBombs");
        // Split on picking up the Power Bomb Pack in Wasteland
        settings.insert_with_parent("shameBombs", false, "specificBombs");
        // Split on picking up the Power Bomb Pack in East Sand Hall
        settings.insert_with_parent("rightSandPitBombs", false, "specificBombs");

        // Split on Varia and Gravity pickups
        settings.insert("suitUpgrades", true);
        // Split on picking up the Varia Suit
        settings.insert_with_parent("variaSuit", true, "suitUpgrades");
        // Split on picking up the Gravity Suit
        settings.insert_with_parent("gravSuit", true, "suitUpgrades");

        // Split on beam upgrades
        settings.insert("beamUpgrades", true);
        // Split on picking up the Charge Beam
        settings.insert_with_parent("chargeBeam", false, "beamUpgrades");
        // Split on picking up the Spazer
        settings.insert_with_parent("spazer", false, "beamUpgrades");
        // Split on picking up the Wave Beam
        settings.insert_with_parent("wave", true, "beamUpgrades");
        // Split on picking up the Ice Beam
        settings.insert_with_parent("ice", false, "beamUpgrades");
        // Split on picking up the Plasma Beam
        settings.insert_with_parent("plasma", false, "beamUpgrades");

        // Split on boot upgrades
        settings.insert("bootUpgrades", false);
        // Split on picking up the Hi-Jump Boots
        settings.insert_with_parent("hiJump", false, "bootUpgrades");
        // Split on picking up Space Jump
        settings.insert_with_parent("spaceJump", false, "bootUpgrades");
        // Split on picking up the Speed Booster
        settings.insert_with_parent("speedBooster", false, "bootUpgrades");

        // Split on Energy Tanks and Reserve Tanks
        settings.insert("energyUpgrades", false);
        // Split on picking up the first Energy Tank
        settings.insert_with_parent("firstETank", false, "energyUpgrades");
        // Split on picking up each Energy Tank
        settings.insert_with_parent("allETanks", false, "energyUpgrades");
        // Split on specific Energy Tank locations
        settings.insert_with_parent("specificETanks", false, "energyUpgrades");
        // Split on picking up the Energy Tank in the Gauntlet Energy Tank Room
        settings.insert_with_parent("gauntletETank", false, "specificETanks");
        // Split on picking up the Energy Tank in the Terminator Room
        settings.insert_with_parent("terminatorETank", false, "specificETanks");
        // Split on picking up the Energy Tank in the Blue Brinstar Energy Tank Room
        settings.insert_with_parent("ceilingETank", false, "specificETanks");
        // Split on picking up the Energy Tank in the Etecoon Energy Tank Room
        settings.insert_with_parent("etecoonsETank", false, "specificETanks");
        // Split on picking up the Energy Tank in Waterway
        settings.insert_with_parent("waterwayETank", false, "specificETanks");
        // Split on picking up the Energy Tank in the Hopper Energy Tank Room
        settings.insert_with_parent("waveGateETank", false, "specificETanks");
        // Split on picking up the Kraid Energy Tank in the Warehouse Energy Tank Room
        settings.insert_with_parent("kraidETank", false, "specificETanks");
        // Split on picking up the Energy Tank in Crocomire's Room
        settings.insert_with_parent("crocomireETank", false, "specificETanks");
        // Split on picking up the Energy Tank in the Hi Jump Energy Tank Room
        settings.insert_with_parent("hiJumpETank", false, "specificETanks");
        // Split on picking up the Energy Tank in the Ridley Tank Room
        settings.insert_with_parent("ridleyETank", false, "specificETanks");
        // Split on picking up the Energy Tank in the Lower Norfair Fireflea Room
        settings.insert_with_parent("firefleaETank", false, "specificETanks");
        // Split on picking up the Energy Tank in the Wrecked Ship Energy Tank Room
        settings.insert_with_parent("wreckedShipETank", false, "specificETanks");
        // Split on picking up the Energy Tank in the Mama Turtle Room
        settings.insert_with_parent("tatoriETank", false, "specificETanks");
        // Split on picking up the Energy Tank in the Botwoon Energy Tank Room
        settings.insert_with_parent("botwoonETank", false, "specificETanks");
        // Split on picking up each Reserve Tank
        settings.insert_with_parent("reserveTanks", false, "energyUpgrades");
        // Split on specific Reserve Tank locations
        settings.insert_with_parent("specificRTanks", false, "energyUpgrades");
        // Split on picking up the Reserve Tank in the Brinstar Reserve Tank Room
        settings.insert_with_parent("brinstarReserve", false, "specificRTanks");
        // Split on picking up the Reserve Tank in the Norfair Reserve Tank Room
        settings.insert_with_parent("norfairReserve", false, "specificRTanks");
        // Split on picking up the Reserve Tank in Bowling Alley
        settings.insert_with_parent("wreckedShipReserve", false, "specificRTanks");
        // Split on picking up the Reserve Tank in West Sand Hole
        settings.insert_with_parent("maridiaReserve", false, "specificRTanks");

        // Split on the miscellaneous upgrades
        settings.insert("miscUpgrades", false);
        // Split on picking up the Morphing Ball
        settings.insert_with_parent("morphBall", false, "miscUpgrades");
        // Split on picking up the Bomb
        settings.insert_with_parent("bomb", false, "miscUpgrades");
        // Split on picking up the Spring Ball
        settings.insert_with_parent("springBall", false, "miscUpgrades");
        // Split on picking up the Screw Attack
        settings.insert_with_parent("screwAttack", false, "miscUpgrades");
        // Split on picking up the Grapple Beam
        settings.insert_with_parent("grapple", false, "miscUpgrades");
        // Split on picking up the X-Ray Scope
        settings.insert_with_parent("xray", false, "miscUpgrades");

        // Split on transitions between areas
        settings.insert("areaTransitions", true);
        // Split on entering miniboss rooms (except Bomb Torizo)
        settings.insert_with_parent("miniBossRooms", false, "areaTransitions");
        // Split on entering major boss rooms
        settings.insert_with_parent("bossRooms", false, "areaTransitions");
        // Split on elevator transitions between areas (except Statue Room to Tourian)
        settings.insert_with_parent("elevatorTransitions", false, "areaTransitions");
        // Split on leaving Ceres Station
        settings.insert_with_parent("ceresEscape", false, "areaTransitions");
        // Split on entering the Wrecked Ship Entrance from the lower door of West Ocean
        settings.insert_with_parent("wreckedShipEntrance", false, "areaTransitions");
        // Split on entering Red Tower from Noob Bridge
        settings.insert_with_parent("redTowerMiddleEntrance", false, "areaTransitions");
        // Split on entering Red Tower from Skree Boost room
        settings.insert_with_parent("redTowerBottomEntrance", false, "areaTransitions");
        // Split on entering Kraid's Lair
        settings.insert_with_parent("kraidsLair", false, "areaTransitions");
        // Split on entering Rising Tide from Cathedral
        settings.insert_with_parent("risingTideEntrance", false, "areaTransitions");
        // Split on exiting Attic
        settings.insert_with_parent("atticExit", false, "areaTransitions");
        // Split on blowing up the tube to enter Maridia
        settings.insert_with_parent("tubeBroken", false, "areaTransitions");
        // Split on exiting West Cacattack Alley
        settings.insert_with_parent("cacExit", false, "areaTransitions");
        // Split on entering Toilet Bowl from either direction
        settings.insert_with_parent("toilet", false, "areaTransitions");
        // Split on entering Kronic Boost room
        settings.insert_with_parent("kronicBoost", false, "areaTransitions");
        // Split on the elevator down to Lower Norfair
        settings.insert_with_parent("lowerNorfairEntrance", false, "areaTransitions");
        // Split on entering Worst Room in the Game
        settings.insert_with_parent("writg", false, "areaTransitions");
        // Split on entering Red Kihunter Shaft from either Amphitheatre or Wastelands (NOTE: will split twice)
        settings.insert_with_parent("redKiShaft", false, "areaTransitions");
        // Split on entering Metal Pirates Room from Wasteland
        settings.insert_with_parent("metalPirates", false, "areaTransitions");
        // Split on entering Lower Norfair Springball Maze Room
        settings.insert_with_parent("lowerNorfairSpringMaze", false, "areaTransitions");
        // Split on moving from the Three Musketeers' Room to the Single Chamber
        settings.insert_with_parent("lowerNorfairExit", false, "areaTransitions");
        // Split on entering the Statues Room with all four major bosses defeated
        settings.insert_with_parent("goldenFour", true, "areaTransitions");
        // Split on the elevator down to Tourian
        settings.insert_with_parent("tourianEntrance", false, "areaTransitions");
        // Split on exiting each of the Metroid rooms in Tourian
        settings.insert_with_parent("metroids", false, "areaTransitions");
        // Split on moving from the Dust Torizo Room to the Big Boy Room
        settings.insert_with_parent("babyMetroidRoom", false, "areaTransitions");
        // Split on moving from Tourian Escape Room 4 to The Climb
        settings.insert_with_parent("escapeClimb", false, "areaTransitions");

        // Split on defeating minibosses
        settings.insert("miniBosses", false);
        // Split on starting the Ceres Escape
        settings.insert_with_parent("ceresRidley", false, "miniBosses");
        // Split on Bomb Torizo's drops appearing
        settings.insert_with_parent("bombTorizo", false, "miniBosses");
        // Split on the last hit to Spore Spawn
        settings.insert_with_parent("sporeSpawn", false, "miniBosses");
        // Split on Crocomire's drops appearing
        settings.insert_with_parent("crocomire", false, "miniBosses");
        // Split on Botwoon's vertical column being fully destroyed
        settings.insert_with_parent("botwoon", false, "miniBosses");
        // Split on Golden Torizo's drops appearing
        settings.insert_with_parent("goldenTorizo", false, "miniBosses");

        // Split on defeating major bosses
        settings.insert("bosses", true);
        // Split shortly after Kraid's drops appear
        settings.insert_with_parent("kraid", false, "bosses");
        // Split on Phantoon's drops appearing
        settings.insert_with_parent("phantoon", false, "bosses");
        // Split on Draygon's drops appearing
        settings.insert_with_parent("draygon", false, "bosses");
        // Split on Ridley's drops appearing
        settings.insert_with_parent("ridley", true, "bosses");
        // Split on Mother Brain's head hitting the ground at the end of the first phase
        settings.insert_with_parent("mb1", false, "bosses");
        // Split on the Baby Metroid detaching from Mother Brain's head
        settings.insert_with_parent("mb2", true, "bosses");
        // Split on the start of the Zebes Escape
        settings.insert_with_parent("mb3", false, "bosses");

        // Split on facing forward at the end of Zebes Escape
        settings.insert("rtaFinish", true);
        // Split on In-Game Time finalizing, when the end cutscene starts
        settings.insert("igtFinish", false);
        // Split on the end of a Spore Spawn RTA run, when the text box clears after collecting the Super Missiles
        settings.insert("sporeSpawnRTAFinish", false);
        // Split on the end of a 100 Missile RTA run, when the text box clears after collecting the hundredth missile
        settings.insert("hundredMissileRTAFinish", false);
        settings.modified_after_creation = false;
        settings
    }

    fn insert(&mut self, name: &str, value: bool) {
        self.modified_after_creation = true;
        self.data.insert(name.into(), (value, None));
    }

    fn insert_with_parent(&mut self, name: &str, value: bool, parent: &str) {
        self.modified_after_creation = true;
        self.data
            .insert(name.into(), (value, Some(parent.to_owned())));
    }

    #[allow(dead_code)]
    fn contains(&self, var: &str) -> bool {
        self.data.contains_key(var)
    }

    fn get(&self, var: &str) -> bool {
        match self.data.get(var) {
            None => false,
            Some((b, None)) => *b,
            Some((b, Some(p))) => *b && self.get(p),
        }
    }

    fn set(&mut self, var: &str, value: bool) {
        let val = match self.data.get_mut(var) {
            None => (value, None),
            Some((_, x)) => (value, x.clone()),
        };
        self.data.insert(std::sync::Arc::from(var), val);
    }

    /// The keys which have no parent defined
    pub fn roots(&self) -> Vec<String> {
        let mut rs = vec![];
        for (key, (_, parent)) in self.data.iter() {
            if parent.is_none() {
                rs.push(key.to_string());
            }
        }
        rs
    }

    /// The immediate childern (if any)
    pub fn children(&self, key: &str) -> Vec<String> {
        let mut rs = vec![];
        for (k, (_, parent)) in self.data.iter() {
            if let Some(parent) = parent {
                if key == parent {
                    rs.push(k.to_string())
                }
            }
        }
        rs
    }

    pub fn lookup(&mut self, var: &str) -> bool {
        match self.data.get(var) {
            None => panic!(),
            Some((b, _)) => *b,
        }
    }

    pub fn lookup_mut(&mut self, var: &str) -> &mut bool {
        // TODO: this is a conservative overapproximation. We don't actually
        // know if the caller wrote to the &mut bool we gave them.
        self.modified_after_creation = true;
        match self.data.get_mut(var) {
            None => panic!(),
            Some((b, _)) => b,
        }
    }

    pub fn has_been_modified(&self) -> bool {
        self.modified_after_creation
    }

    pub fn split_on_misc_upgrades(&mut self) {
        self.set("miscUpgrades", true);
        self.set("morphBall", true);
        self.set("bomb", true);
        self.set("springBall", true);
        self.set("screwAttack", true);
        self.set("grapple", true);
        self.set("xray", true);
    }

    pub fn split_on_hundo(&mut self) {
        self.set("ammoPickups", true);
        self.set("allMissiles", true);
        self.set("allSupers", true);
        self.set("allPowerBombs", true);
        self.set("beamUpgrades", true);
        self.set("chargeBeam", true);
        self.set("spazer", true);
        self.set("wave", true);
        self.set("ice", true);
        self.set("plasma", true);
        self.set("bootUpgrades", true);
        self.set("hiJump", true);
        self.set("spaceJump", true);
        self.set("speedBooster", true);
        self.set("energyUpgrades", true);
        self.set("allETanks", true);
        self.set("reserveTanks", true);
        self.split_on_misc_upgrades();
        self.set("areaTransitions", true); // should already be true
        self.set("tubeBroken", true);
        self.set("ceresEscape", true);
        self.set("bosses", true); // should already be true
        self.set("kraid", true);
        self.set("phantoon", true);
        self.set("draygon", true);
        self.set("ridley", true);
        self.set("mb1", true);
        self.set("mb2", true);
        self.set("mb3", true);
        self.set("miniBosses", true);
        self.set("ceresRidley", true);
        self.set("bombTorizo", true);
        self.set("crocomire", true);
        self.set("botwoon", true);
        self.set("goldenTorizo", true);
        self.set("babyMetroidRoom", true);
    }
    pub fn split_on_anypercent(&mut self) {
        self.set("ammoPickups", true);
        self.set("specificMissiles", true);
        self.set("specificSupers", true);
        self.set("wreckedShipLeftSupers", true);
        self.set("specificPowerBombs", true);
        self.set("firstMissile", true);
        self.set("firstSuper", true);
        self.set("firstPowerBomb", true);
        self.set("brinstarMissiles", true);
        self.set("norfairMissiles", true);
        self.set("chargeMissiles", true);
        self.set("waveMissiles", true);
        self.set("beamUpgrades", true);
        self.set("chargeBeam", true);
        self.set("wave", true);
        self.set("ice", true);
        self.set("plasma", true);
        self.set("bootUpgrades", true);
        self.set("hiJump", true);
        self.set("speedBooster", true);
        self.set("specificETanks", true);
        self.set("energyUpgrades", true);
        self.set("terminatorETank", true);
        self.set("hiJumpETank", true);
        self.set("botwoonETank", true);
        self.set("miscUpgrades", true);
        self.set("morphBall", true);
        self.set("spaceJump", true);
        self.set("bomb", true);
        self.set("areaTransitions", true); // should already be true
        self.set("tubeBroken", true);
        self.set("ceresEscape", true);
        self.set("bosses", true); // should already be true
        self.set("kraid", true);
        self.set("phantoon", true);
        self.set("draygon", true);
        self.set("ridley", true);
        self.set("mb1", true);
        self.set("mb2", true);
        self.set("mb3", true);
        self.set("miniBosses", true);
        self.set("ceresRidley", true);
        self.set("bombTorizo", true);
        self.set("botwoon", true);
        self.set("goldenTorizo", true);
        self.set("babyMetroidRoom", true);
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(non_snake_case)]
#[allow(clippy::all)]
// TODO: probably makes sense to move this to the SNESState impl
fn split(settings: &Settings, snes: &mut SNESState) -> bool {
    // Ammo pickup section
    let firstMissile = settings.get("firstMissile")
        && snes["maxMissiles"].old == 0
        && snes["maxMissiles"].current == 5;
    let allMissiles = settings.get("allMissiles")
        && (snes["maxMissiles"].old + 5) == (snes["maxMissiles"].current);
    let oceanBottomMissiles = settings.get("oceanBottomMissiles")
        && snes["roomID"].current == roomIDEnum["westOcean"]
        && (snes["crateriaItems"].old + 2) == (snes["crateriaItems"].current);
    let oceanTopMissiles = settings.get("oceanTopMissiles")
        && snes["roomID"].current == roomIDEnum["westOcean"]
        && (snes["crateriaItems"].old + 4) == (snes["crateriaItems"].current);
    let oceanMiddleMissiles = settings.get("oceanMiddleMissiles")
        && snes["roomID"].current == roomIDEnum["westOcean"]
        && (snes["crateriaItems"].old + 8) == (snes["crateriaItems"].current);
    let moatMissiles = settings.get("moatMissiles")
        && snes["roomID"].current == roomIDEnum["crateriaMoat"]
        && (snes["crateriaItems"].old + 16) == (snes["crateriaItems"].current);
    let oldTourianMissiles = settings.get("oldTourianMissiles")
        && snes["roomID"].current == roomIDEnum["pitRoom"]
        && (snes["crateriaItems"].old + 64) == (snes["crateriaItems"].current);
    let gauntletRightMissiles = settings.get("gauntletRightMissiles")
        && snes["roomID"].current == roomIDEnum["greenPirateShaft"]
        && (snes["brinteriaItems"].old + 2) == (snes["brinteriaItems"].current);
    let gauntletLeftMissiles = settings.get("gauntletLeftMissiles")
        && snes["roomID"].current == roomIDEnum["greenPirateShaft"]
        && (snes["brinteriaItems"].old + 4) == (snes["brinteriaItems"].current);
    let dentalPlan = settings.get("dentalPlan")
        && snes["roomID"].current == roomIDEnum["theFinalMissile"]
        && (snes["brinteriaItems"].old + 16) == (snes["brinteriaItems"].current);
    let earlySuperBridgeMissiles = settings.get("earlySuperBridgeMissiles")
        && snes["roomID"].current == roomIDEnum["earlySupers"]
        && (snes["brinteriaItems"].old + 128) == (snes["brinteriaItems"].current);
    let greenBrinstarReserveMissiles = settings.get("greenBrinstarReserveMissiles")
        && snes["roomID"].current == roomIDEnum["brinstarReserveRoom"]
        && (snes["brinstarItems2"].old + 8) == (snes["brinstarItems2"].current);
    let greenBrinstarExtraReserveMissiles = settings.get("greenBrinstarExtraReserveMissiles")
        && snes["roomID"].current == roomIDEnum["brinstarReserveRoom"]
        && (snes["brinstarItems2"].old + 4) == (snes["brinstarItems2"].current);
    let bigPinkTopMissiles = settings.get("bigPinkTopMissiles")
        && snes["roomID"].current == roomIDEnum["bigPink"]
        && (snes["brinstarItems2"].old + 32) == (snes["brinstarItems2"].current);
    let chargeMissiles = settings.get("chargeMissiles")
        && snes["roomID"].current == roomIDEnum["bigPink"]
        && (snes["brinstarItems2"].old + 64) == (snes["brinstarItems2"].current);
    let greenHillsMissiles = settings.get("greenHillsMissiles")
        && snes["roomID"].current == roomIDEnum["greenHills"]
        && (snes["brinstarItems3"].old + 2) == (snes["brinstarItems3"].current);
    let blueBrinstarETankMissiles = settings.get("blueBrinstarETankMissiles")
        && snes["roomID"].current == roomIDEnum["blueBrinstarETankRoom"]
        && (snes["brinstarItems3"].old + 16) == (snes["brinstarItems3"].current);
    let alphaMissiles = settings.get("alphaMissiles")
        && snes["roomID"].current == roomIDEnum["alphaMissileRoom"]
        && (snes["brinstarItems4"].old + 4) == (snes["brinstarItems4"].current);
    let billyMaysMissiles = settings.get("billyMaysMissiles")
        && snes["roomID"].current == roomIDEnum["billyMays"]
        && (snes["brinstarItems4"].old + 16) == (snes["brinstarItems4"].current);
    let butWaitTheresMoreMissiles = settings.get("butWaitTheresMoreMissiles")
        && snes["roomID"].current == roomIDEnum["billyMays"]
        && (snes["brinstarItems4"].old + 32) == (snes["brinstarItems4"].current);
    let redBrinstarMissiles = settings.get("redBrinstarMissiles")
        && snes["roomID"].current == roomIDEnum["alphaPowerBombsRoom"]
        && (snes["brinstarItems5"].old + 2) == (snes["brinstarItems5"].current);
    let warehouseMissiles = settings.get("warehouseMissiles")
        && snes["roomID"].current == roomIDEnum["warehouseKiHunters"]
        && (snes["brinstarItems5"].old + 16) == (snes["brinstarItems5"].current);
    let cathedralMissiles = settings.get("cathedralMissiles")
        && snes["roomID"].current == roomIDEnum["cathedral"]
        && (snes["norfairItems1"].old + 2) == (snes["norfairItems1"].current);
    let crumbleShaftMissiles = settings.get("crumbleShaftMissiles")
        && snes["roomID"].current == roomIDEnum["crumbleShaft"]
        && (snes["norfairItems1"].old + 8) == (snes["norfairItems1"].current);
    let crocomireEscapeMissiles = settings.get("crocomireEscapeMissiles")
        && snes["roomID"].current == roomIDEnum["crocomireEscape"]
        && (snes["norfairItems1"].old + 64) == (snes["norfairItems1"].current);
    let hiJumpMissiles = settings.get("hiJumpMissiles")
        && snes["roomID"].current == roomIDEnum["hiJumpShaft"]
        && (snes["norfairItems1"].old + 128) == (snes["norfairItems1"].current);
    let postCrocomireMissiles = settings.get("postCrocomireMissiles")
        && snes["roomID"].current == roomIDEnum["cosineRoom"]
        && (snes["norfairItems2"].old + 4) == (snes["norfairItems2"].current);
    let grappleMissiles = settings.get("grappleMissiles")
        && snes["roomID"].current == roomIDEnum["preGrapple"]
        && (snes["norfairItems2"].old + 8) == (snes["norfairItems2"].current);
    let norfairReserveMissiles = settings.get("norfairReserveMissiles")
        && snes["roomID"].current == roomIDEnum["norfairReserveRoom"]
        && (snes["norfairItems2"].old + 64) == (snes["norfairItems2"].current);
    let greenBubblesMissiles = settings.get("greenBubblesMissiles")
        && snes["roomID"].current == roomIDEnum["greenBubblesRoom"]
        && (snes["norfairItems2"].old + 128) == (snes["norfairItems2"].current);
    let bubbleMountainMissiles = settings.get("bubbleMountainMissiles")
        && snes["roomID"].current == roomIDEnum["bubbleMountain"]
        && (snes["norfairItems3"].old + 1) == (snes["norfairItems3"].current);
    let speedBoostMissiles = settings.get("speedBoostMissiles")
        && snes["roomID"].current == roomIDEnum["speedBoostHall"]
        && (snes["norfairItems3"].old + 2) == (snes["norfairItems3"].current);
    let waveMissiles = settings.get("waveMissiles")
        && snes["roomID"].current == roomIDEnum["doubleChamber"]
        && (snes["norfairItems3"].old + 8) == (snes["norfairItems3"].current);
    let goldTorizoMissiles = settings.get("goldTorizoMissiles")
        && snes["roomID"].current == roomIDEnum["goldenTorizo"]
        && (snes["norfairItems3"].old + 64) == (snes["norfairItems3"].current);
    let mickeyMouseMissiles = settings.get("mickeyMouseMissiles")
        && snes["roomID"].current == roomIDEnum["mickeyMouse"]
        && (snes["norfairItems4"].old + 2) == (snes["norfairItems4"].current);
    let lowerNorfairSpringMazeMissiles = settings.get("lowerNorfairSpringMazeMissiles")
        && snes["roomID"].current == roomIDEnum["lowerNorfairSpringMaze"]
        && (snes["norfairItems4"].old + 4) == (snes["norfairItems4"].current);
    let threeMusketeersMissiles = settings.get("threeMusketeersMissiles")
        && snes["roomID"].current == roomIDEnum["threeMusketeers"]
        && (snes["norfairItems4"].old + 32) == (snes["norfairItems4"].current);
    let wreckedShipMainShaftMissiles = settings.get("wreckedShipMainShaftMissiles")
        && snes["roomID"].current == roomIDEnum["wreckedShipMainShaft"]
        && (snes["wreckedShipItems"].old + 1) == (snes["wreckedShipItems"].current);
    let bowlingMissiles = settings.get("bowlingMissiles")
        && snes["roomID"].current == roomIDEnum["bowling"]
        && (snes["wreckedShipItems"].old + 4) == (snes["wreckedShipItems"].current);
    let atticMissiles = settings.get("atticMissiles")
        && snes["roomID"].current == roomIDEnum["atticWorkerRobotRoom"]
        && (snes["wreckedShipItems"].old + 8) == (snes["wreckedShipItems"].current);
    let mainStreetMissiles = settings.get("mainStreetMissiles")
        && snes["roomID"].current == roomIDEnum["mainStreet"]
        && (snes["maridiaItems1"].old + 1) == (snes["maridiaItems1"].current);
    let mamaTurtleMissiles = settings.get("mamaTurtleMissiles")
        && snes["roomID"].current == roomIDEnum["mamaTurtle"]
        && (snes["maridiaItems1"].old + 8) == (snes["maridiaItems1"].current);
    let wateringHoleMissiles = settings.get("wateringHoleMissiles")
        && snes["roomID"].current == roomIDEnum["wateringHole"]
        && (snes["maridiaItems1"].old + 32) == (snes["maridiaItems1"].current);
    let beachMissiles = settings.get("beachMissiles")
        && snes["roomID"].current == roomIDEnum["beach"]
        && (snes["maridiaItems1"].old + 64) == (snes["maridiaItems1"].current);
    let leftSandPitMissiles = settings.get("leftSandPitMissiles")
        && snes["roomID"].current == roomIDEnum["leftSandPit"]
        && (snes["maridiaItems2"].old + 1) == (snes["maridiaItems2"].current);
    let rightSandPitMissiles = settings.get("rightSandPitMissiles")
        && snes["roomID"].current == roomIDEnum["rightSandPit"]
        && (snes["maridiaItems2"].old + 4) == (snes["maridiaItems2"].current);
    let aqueductMissiles = settings.get("aqueductMissiles")
        && snes["roomID"].current == roomIDEnum["aqueduct"]
        && (snes["maridiaItems2"].old + 16) == (snes["maridiaItems2"].current);
    let preDraygonMissiles = settings.get("preDraygonMissiles")
        && snes["roomID"].current == roomIDEnum["precious"]
        && (snes["maridiaItems2"].old + 128) == (snes["maridiaItems2"].current);
    let firstSuper =
        settings.get("firstSuper") && snes["maxSupers"].old == 0 && snes["maxSupers"].current == 5;
    let allSupers =
        settings.get("allSupers") && (snes["maxSupers"].old + 5) == (snes["maxSupers"].current);
    let climbSupers = settings.get("climbSupers")
        && snes["roomID"].current == roomIDEnum["crateriaSupersRoom"]
        && (snes["brinteriaItems"].old + 8) == (snes["brinteriaItems"].current);
    let sporeSpawnSupers = settings.get("sporeSpawnSupers")
        && snes["roomID"].current == roomIDEnum["sporeSpawnSuper"]
        && (snes["brinteriaItems"].old + 64) == (snes["brinteriaItems"].current);
    let earlySupers = settings.get("earlySupers")
        && snes["roomID"].current == roomIDEnum["earlySupers"]
        && (snes["brinstarItems2"].old + 1) == (snes["brinstarItems2"].current);
    let etecoonSupers = (settings.get("etecoonSupers") || settings.get("etacoonSupers"))
        && snes["roomID"].current == roomIDEnum["etecoonSuperRoom"]
        && (snes["brinstarItems3"].old + 128) == (snes["brinstarItems3"].current);
    let goldTorizoSupers = settings.get("goldTorizoSupers")
        && snes["roomID"].current == roomIDEnum["goldenTorizo"]
        && (snes["norfairItems3"].old + 128) == (snes["norfairItems3"].current);
    let wreckedShipLeftSupers = settings.get("wreckedShipLeftSupers")
        && snes["roomID"].current == roomIDEnum["wreckedShipLeftSuperRoom"]
        && (snes["wreckedShipItems"].old + 32) == (snes["wreckedShipItems"].current);
    let wreckedShipRightSupers = settings.get("wreckedShipRightSupers")
        && snes["roomID"].current == roomIDEnum["wreckedShipRightSuperRoom"]
        && (snes["wreckedShipItems"].old + 64) == (snes["wreckedShipItems"].current);
    let crabSupers = settings.get("crabSupers")
        && snes["roomID"].current == roomIDEnum["mainStreet"]
        && (snes["maridiaItems1"].old + 2) == (snes["maridiaItems1"].current);
    let wateringHoleSupers = settings.get("wateringHoleSupers")
        && snes["roomID"].current == roomIDEnum["wateringHole"]
        && (snes["maridiaItems1"].old + 16) == (snes["maridiaItems1"].current);
    let aqueductSupers = settings.get("aqueductSupers")
        && snes["roomID"].current == roomIDEnum["aqueduct"]
        && (snes["maridiaItems2"].old + 32) == (snes["maridiaItems2"].current);
    let firstPowerBomb = settings.get("firstPowerBomb")
        && snes["maxPowerBombs"].old == 0
        && snes["maxPowerBombs"].current == 5;
    let allPowerBombs = settings.get("allPowerBombs")
        && (snes["maxPowerBombs"].old + 5) == (snes["maxPowerBombs"].current);
    let landingSiteBombs = settings.get("landingSiteBombs")
        && snes["roomID"].current == roomIDEnum["crateriaPowerBombRoom"]
        && (snes["crateriaItems"].old + 1) == (snes["crateriaItems"].current);
    let etecoonBombs = (settings.get("etecoonBombs") || settings.get("etacoonBombs"))
        && snes["roomID"].current == roomIDEnum["greenBrinstarMainShaft"]
        && (snes["brinteriaItems"].old + 32) == (snes["brinteriaItems"].current);
    let pinkBrinstarBombs = settings.get("pinkBrinstarBombs")
        && snes["roomID"].current == roomIDEnum["pinkBrinstarPowerBombRoom"]
        && (snes["brinstarItems3"].old + 1) == (snes["brinstarItems3"].current);
    let blueBrinstarBombs = settings.get("blueBrinstarBombs")
        && snes["roomID"].current == roomIDEnum["morphBall"]
        && (snes["brinstarItems3"].old + 8) == (snes["brinstarItems3"].current);
    let alphaBombs = settings.get("alphaBombs")
        && snes["roomID"].current == roomIDEnum["alphaPowerBombsRoom"]
        && (snes["brinstarItems5"].old + 1) == (snes["brinstarItems5"].current);
    let betaBombs = settings.get("betaBombs")
        && snes["roomID"].current == roomIDEnum["betaPowerBombRoom"]
        && (snes["brinstarItems4"].old + 128) == (snes["brinstarItems4"].current);
    let crocomireBombs = settings.get("crocomireBombs")
        && snes["roomID"].current == roomIDEnum["postCrocomirePowerBombRoom"]
        && (snes["norfairItems2"].old + 2) == (snes["norfairItems2"].current);
    let lowerNorfairEscapeBombs = settings.get("lowerNorfairEscapeBombs")
        && snes["roomID"].current == roomIDEnum["lowerNorfairEscapePowerBombRoom"]
        && (snes["norfairItems4"].old + 8) == (snes["norfairItems4"].current);
    let shameBombs = settings.get("shameBombs")
        && snes["roomID"].current == roomIDEnum["wasteland"]
        && (snes["norfairItems4"].old + 16) == (snes["norfairItems4"].current);
    let rightSandPitBombs = settings.get("rightSandPitBombs")
        && snes["roomID"].current == roomIDEnum["rightSandPit"]
        && (snes["maridiaItems2"].old + 8) == (snes["maridiaItems2"].current);
    let pickup = firstMissile
        || allMissiles
        || oceanBottomMissiles
        || oceanTopMissiles
        || oceanMiddleMissiles
        || moatMissiles
        || oldTourianMissiles
        || gauntletRightMissiles
        || gauntletLeftMissiles
        || dentalPlan
        || earlySuperBridgeMissiles
        || greenBrinstarReserveMissiles
        || greenBrinstarExtraReserveMissiles
        || bigPinkTopMissiles
        || chargeMissiles
        || greenHillsMissiles
        || blueBrinstarETankMissiles
        || alphaMissiles
        || billyMaysMissiles
        || butWaitTheresMoreMissiles
        || redBrinstarMissiles
        || warehouseMissiles
        || cathedralMissiles
        || crumbleShaftMissiles
        || crocomireEscapeMissiles
        || hiJumpMissiles
        || postCrocomireMissiles
        || grappleMissiles
        || norfairReserveMissiles
        || greenBubblesMissiles
        || bubbleMountainMissiles
        || speedBoostMissiles
        || waveMissiles
        || goldTorizoMissiles
        || mickeyMouseMissiles
        || lowerNorfairSpringMazeMissiles
        || threeMusketeersMissiles
        || wreckedShipMainShaftMissiles
        || bowlingMissiles
        || atticMissiles
        || mainStreetMissiles
        || mamaTurtleMissiles
        || wateringHoleMissiles
        || beachMissiles
        || leftSandPitMissiles
        || rightSandPitMissiles
        || aqueductMissiles
        || preDraygonMissiles
        || firstSuper
        || allSupers
        || climbSupers
        || sporeSpawnSupers
        || earlySupers
        || etecoonSupers
        || goldTorizoSupers
        || wreckedShipLeftSupers
        || wreckedShipRightSupers
        || crabSupers
        || wateringHoleSupers
        || aqueductSupers
        || firstPowerBomb
        || allPowerBombs
        || landingSiteBombs
        || etecoonBombs
        || pinkBrinstarBombs
        || blueBrinstarBombs
        || alphaBombs
        || betaBombs
        || crocomireBombs
        || lowerNorfairEscapeBombs
        || shameBombs
        || rightSandPitBombs;

    // Item unlock section
    let varia = settings.get("variaSuit")
        && snes["roomID"].current == roomIDEnum["varia"]
        && (snes["unlockedEquips2"].old & unlockFlagEnum["variaSuit"]) == 0
        && (snes["unlockedEquips2"].current & unlockFlagEnum["variaSuit"]) > 0;
    let springBall = settings.get("springBall")
        && snes["roomID"].current == roomIDEnum["springBall"]
        && (snes["unlockedEquips2"].old & unlockFlagEnum["springBall"]) == 0
        && (snes["unlockedEquips2"].current & unlockFlagEnum["springBall"]) > 0;
    let morphBall = settings.get("morphBall")
        && snes["roomID"].current == roomIDEnum["morphBall"]
        && (snes["unlockedEquips2"].old & unlockFlagEnum["morphBall"]) == 0
        && (snes["unlockedEquips2"].current & unlockFlagEnum["morphBall"]) > 0;
    let screwAttack = settings.get("screwAttack")
        && snes["roomID"].current == roomIDEnum["screwAttack"]
        && (snes["unlockedEquips2"].old & unlockFlagEnum["screwAttack"]) == 0
        && (snes["unlockedEquips2"].current & unlockFlagEnum["screwAttack"]) > 0;
    let gravSuit = settings.get("gravSuit")
        && snes["roomID"].current == roomIDEnum["gravity"]
        && (snes["unlockedEquips2"].old & unlockFlagEnum["gravSuit"]) == 0
        && (snes["unlockedEquips2"].current & unlockFlagEnum["gravSuit"]) > 0;
    let hiJump = settings.get("hiJump")
        && snes["roomID"].current == roomIDEnum["hiJump"]
        && (snes["unlockedEquips"].old & unlockFlagEnum["hiJump"]) == 0
        && (snes["unlockedEquips"].current & unlockFlagEnum["hiJump"]) > 0;
    let spaceJump = settings.get("spaceJump")
        && snes["roomID"].current == roomIDEnum["spaceJump"]
        && (snes["unlockedEquips"].old & unlockFlagEnum["spaceJump"]) == 0
        && (snes["unlockedEquips"].current & unlockFlagEnum["spaceJump"]) > 0;
    let bomb = settings.get("bomb")
        && snes["roomID"].current == roomIDEnum["bombTorizo"]
        && (snes["unlockedEquips"].old & unlockFlagEnum["bomb"]) == 0
        && (snes["unlockedEquips"].current & unlockFlagEnum["bomb"]) > 0;
    let speedBooster = settings.get("speedBooster")
        && snes["roomID"].current == roomIDEnum["speedBooster"]
        && (snes["unlockedEquips"].old & unlockFlagEnum["speedBooster"]) == 0
        && (snes["unlockedEquips"].current & unlockFlagEnum["speedBooster"]) > 0;
    let grapple = settings.get("grapple")
        && snes["roomID"].current == roomIDEnum["grapple"]
        && (snes["unlockedEquips"].old & unlockFlagEnum["grapple"]) == 0
        && (snes["unlockedEquips"].current & unlockFlagEnum["grapple"]) > 0;
    let xray = settings.get("xray")
        && snes["roomID"].current == roomIDEnum["xRay"]
        && (snes["unlockedEquips"].old & unlockFlagEnum["xray"]) == 0
        && (snes["unlockedEquips"].current & unlockFlagEnum["xray"]) > 0;
    let unlock = varia
        || springBall
        || morphBall
        || screwAttack
        || gravSuit
        || hiJump
        || spaceJump
        || bomb
        || speedBooster
        || grapple
        || xray;

    // Beam unlock section
    let wave = settings.get("wave")
        && snes["roomID"].current == roomIDEnum["waveBeam"]
        && (snes["unlockedBeams"].old & unlockFlagEnum["wave"]) == 0
        && (snes["unlockedBeams"].current & unlockFlagEnum["wave"]) > 0;
    let ice = settings.get("ice")
        && snes["roomID"].current == roomIDEnum["iceBeam"]
        && (snes["unlockedBeams"].old & unlockFlagEnum["ice"]) == 0
        && (snes["unlockedBeams"].current & unlockFlagEnum["ice"]) > 0;
    let spazer = settings.get("spazer")
        && snes["roomID"].current == roomIDEnum["spazer"]
        && (snes["unlockedBeams"].old & unlockFlagEnum["spazer"]) == 0
        && (snes["unlockedBeams"].current & unlockFlagEnum["spazer"]) > 0;
    let plasma = settings.get("plasma")
        && snes["roomID"].current == roomIDEnum["plasmaBeam"]
        && (snes["unlockedBeams"].old & unlockFlagEnum["plasma"]) == 0
        && (snes["unlockedBeams"].current & unlockFlagEnum["plasma"]) > 0;
    let chargeBeam = settings.get("chargeBeam")
        && snes["roomID"].current == roomIDEnum["bigPink"]
        && (snes["unlockedCharge"].old & unlockFlagEnum["chargeBeam"]) == 0
        && (snes["unlockedCharge"].current & unlockFlagEnum["chargeBeam"]) > 0;
    let beam = wave || ice || spazer || plasma || chargeBeam;

    // E-tanks and reserve tanks
    let firstETank = settings.get("firstETank")
        && snes["maxEnergy"].old == 99
        && snes["maxEnergy"].current == 199;
    let allETanks =
        settings.get("allETanks") && (snes["maxEnergy"].old + 100) == (snes["maxEnergy"].current);
    let gauntletETank = settings.get("gauntletETank")
        && snes["roomID"].current == roomIDEnum["gauntletETankRoom"]
        && (snes["crateriaItems"].old + 32) == (snes["crateriaItems"].current);
    let terminatorETank = settings.get("terminatorETank")
        && snes["roomID"].current == roomIDEnum["terminator"]
        && (snes["brinteriaItems"].old + 1) == (snes["brinteriaItems"].current);
    let ceilingETank = settings.get("ceilingETank")
        && snes["roomID"].current == roomIDEnum["blueBrinstarETankRoom"]
        && (snes["brinstarItems3"].old + 32) == (snes["brinstarItems3"].current);
    let etecoonsETank = (settings.get("etecoonsETank") || settings.get("etacoonsETank"))
        && snes["roomID"].current == roomIDEnum["etecoonETankRoom"]
        && (snes["brinstarItems3"].old + 64) == (snes["brinstarItems3"].current);
    let waterwayETank = settings.get("waterwayETank")
        && snes["roomID"].current == roomIDEnum["waterway"]
        && (snes["brinstarItems4"].old + 2) == (snes["brinstarItems4"].current);
    let waveGateETank = settings.get("waveGateETank")
        && snes["roomID"].current == roomIDEnum["hopperETankRoom"]
        && (snes["brinstarItems4"].old + 8) == (snes["brinstarItems4"].current);
    let kraidETank = settings.get("kraidETank")
        && snes["roomID"].current == roomIDEnum["warehouseETankRoom"]
        && (snes["brinstarItems5"].old + 8) == (snes["brinstarItems5"].current);
    let crocomireETank = settings.get("crocomireETank")
        && snes["roomID"].current == roomIDEnum["crocomire"]
        && (snes["norfairItems1"].old + 16) == (snes["norfairItems1"].current);
    let hiJumpETank = settings.get("hiJumpETank")
        && snes["roomID"].current == roomIDEnum["hiJumpShaft"]
        && (snes["norfairItems2"].old + 1) == (snes["norfairItems2"].current);
    let ridleyETank = settings.get("ridleyETank")
        && snes["roomID"].current == roomIDEnum["ridleyETankRoom"]
        && (snes["norfairItems4"].old + 64) == (snes["norfairItems4"].current);
    let firefleaETank = settings.get("firefleaETank")
        && snes["roomID"].current == roomIDEnum["lowerNorfairFireflea"]
        && (snes["norfairItems5"].old + 1) == (snes["norfairItems5"].current);
    let wreckedShipETank = settings.get("wreckedShipETank")
        && snes["roomID"].current == roomIDEnum["wreckedShipETankRoom"]
        && (snes["wreckedShipItems"].old + 16) == (snes["wreckedShipItems"].current);
    let tatoriETank = settings.get("tatoriETank")
        && snes["roomID"].current == roomIDEnum["mamaTurtle"]
        && (snes["maridiaItems1"].old + 4) == (snes["maridiaItems1"].current);
    let botwoonETank = settings.get("botwoonETank")
        && snes["roomID"].current == roomIDEnum["botwoonETankRoom"]
        && (snes["maridiaItems3"].old + 1) == (snes["maridiaItems3"].current);
    let reserveTanks = settings.get("reserveTanks")
        && (snes["maxReserve"].old + 100) == (snes["maxReserve"].current);
    let brinstarReserve = settings.get("brinstarReserve")
        && snes["roomID"].current == roomIDEnum["brinstarReserveRoom"]
        && (snes["brinstarItems2"].old + 2) == (snes["brinstarItems2"].current);
    let norfairReserve = settings.get("norfairReserve")
        && snes["roomID"].current == roomIDEnum["norfairReserveRoom"]
        && (snes["norfairItems2"].old + 32) == (snes["norfairItems2"].current);
    let wreckedShipReserve = settings.get("wreckedShipReserve")
        && snes["roomID"].current == roomIDEnum["bowling"]
        && (snes["wreckedShipItems"].old + 2) == (snes["wreckedShipItems"].current);
    let maridiaReserve = settings.get("maridiaReserve")
        && snes["roomID"].current == roomIDEnum["leftSandPit"]
        && (snes["maridiaItems2"].old + 2) == (snes["maridiaItems2"].current);
    let energyUpgrade = firstETank
        || allETanks
        || gauntletETank
        || terminatorETank
        || ceilingETank
        || etecoonsETank
        || waterwayETank
        || waveGateETank
        || kraidETank
        || crocomireETank
        || hiJumpETank
        || ridleyETank
        || firefleaETank
        || wreckedShipETank
        || tatoriETank
        || botwoonETank
        || reserveTanks
        || brinstarReserve
        || norfairReserve
        || wreckedShipReserve
        || maridiaReserve;

    // Miniboss room transitions
    let mut miniBossRooms = false;
    if settings.get("miniBossRooms") {
        let ceresRidleyRoom = snes["roomID"].old == roomIDEnum["flatRoom"]
            && snes["roomID"].current == roomIDEnum["ceresRidley"];
        let sporeSpawnRoom = snes["roomID"].old == roomIDEnum["sporeSpawnKeyhunter"]
            && snes["roomID"].current == roomIDEnum["sporeSpawn"];
        let crocomireRoom = snes["roomID"].old == roomIDEnum["crocomireSpeedway"]
            && snes["roomID"].current == roomIDEnum["crocomire"];
        let botwoonRoom = snes["roomID"].old == roomIDEnum["botwoonHallway"]
            && snes["roomID"].current == roomIDEnum["botwoon"];
        // Allow either vanilla or GGG entry
        let goldenTorizoRoom = (snes["roomID"].old == roomIDEnum["acidStatue"]
            || snes["roomID"].old == roomIDEnum["screwAttack"])
            && snes["roomID"].current == roomIDEnum["goldenTorizo"];
        miniBossRooms =
            ceresRidleyRoom || sporeSpawnRoom || crocomireRoom || botwoonRoom || goldenTorizoRoom;
    }

    // Boss room transitions
    let mut bossRooms = false;
    if settings.get("bossRooms") {
        let kraidRoom = snes["roomID"].old == roomIDEnum["kraidEyeDoor"]
            && snes["roomID"].current == roomIDEnum["kraid"];
        let phantoonRoom = snes["roomID"].old == roomIDEnum["basement"]
            && snes["roomID"].current == roomIDEnum["phantoon"];
        let draygonRoom = snes["roomID"].old == roomIDEnum["precious"]
            && snes["roomID"].current == roomIDEnum["draygon"];
        let ridleyRoom = snes["roomID"].old == roomIDEnum["lowerNorfairFarming"]
            && snes["roomID"].current == roomIDEnum["ridley"];
        let motherBrainRoom = snes["roomID"].old == roomIDEnum["rinkaShaft"]
            && snes["roomID"].current == roomIDEnum["motherBrain"];
        bossRooms = kraidRoom || phantoonRoom || draygonRoom || ridleyRoom || motherBrainRoom;
    }

    // Elevator transitions between areas
    let mut elevatorTransitions = false;
    if settings.get("elevatorTransitions") {
        let blueBrinstar = (snes["roomID"].old == roomIDEnum["elevatorToMorphBall"]
            && snes["roomID"].current == roomIDEnum["morphBall"])
            || (snes["roomID"].old == roomIDEnum["morphBall"]
                && snes["roomID"].current == roomIDEnum["elevatorToMorphBall"]);
        let greenBrinstar = (snes["roomID"].old == roomIDEnum["elevatorToGreenBrinstar"]
            && snes["roomID"].current == roomIDEnum["greenBrinstarMainShaft"])
            || (snes["roomID"].old == roomIDEnum["greenBrinstarMainShaft"]
                && snes["roomID"].current == roomIDEnum["elevatorToGreenBrinstar"]);
        let businessCenter = (snes["roomID"].old == roomIDEnum["warehouseEntrance"]
            && snes["roomID"].current == roomIDEnum["businessCenter"])
            || (snes["roomID"].old == roomIDEnum["businessCenter"]
                && snes["roomID"].current == roomIDEnum["warehouseEntrance"]);
        let caterpillar = (snes["roomID"].old == roomIDEnum["elevatorToCaterpillar"]
            && snes["roomID"].current == roomIDEnum["caterpillar"])
            || (snes["roomID"].old == roomIDEnum["caterpillar"]
                && snes["roomID"].current == roomIDEnum["elevatorToCaterpillar"]);
        let maridiaElevator = (snes["roomID"].old == roomIDEnum["elevatorToMaridia"]
            && snes["roomID"].current == roomIDEnum["maridiaElevator"])
            || (snes["roomID"].old == roomIDEnum["maridiaElevator"]
                && snes["roomID"].current == roomIDEnum["elevatorToMaridia"]);
        elevatorTransitions =
            blueBrinstar || greenBrinstar || businessCenter || caterpillar || maridiaElevator;
    }

    // Room transitions
    let ceresEscape = settings.get("ceresEscape")
        && snes["roomID"].current == roomIDEnum["ceresElevator"]
        && snes["gameState"].old == gameStateEnum["normalGameplay"]
        && snes["gameState"].current == gameStateEnum["startOfCeresCutscene"];
    let wreckedShipEntrance = settings.get("wreckedShipEntrance")
        && snes["roomID"].old == roomIDEnum["westOcean"]
        && snes["roomID"].current == roomIDEnum["wreckedShipEntrance"];
    let redTowerMiddleEntrance = settings.get("redTowerMiddleEntrance")
        && snes["roomID"].old == roomIDEnum["noobBridge"]
        && snes["roomID"].current == roomIDEnum["redTower"];
    let redTowerBottomEntrance = settings.get("redTowerBottomEntrance")
        && snes["roomID"].old == roomIDEnum["bat"]
        && snes["roomID"].current == roomIDEnum["redTower"];
    let kraidsLair = settings.get("kraidsLair")
        && snes["roomID"].old == roomIDEnum["warehouseEntrance"]
        && snes["roomID"].current == roomIDEnum["warehouseZeela"];
    let risingTideEntrance = settings.get("risingTideEntrance")
        && snes["roomID"].old == roomIDEnum["cathedral"]
        && snes["roomID"].current == roomIDEnum["risingTide"];
    let atticExit = settings.get("atticExit")
        && snes["roomID"].old == roomIDEnum["attic"]
        && snes["roomID"].current == roomIDEnum["westOcean"];
    let tubeBroken = settings.get("tubeBroken")
        && snes["roomID"].current == roomIDEnum["glassTunnel"]
        && (snes["eventFlags"].old & eventFlagEnum["tubeBroken"]) == 0
        && (snes["eventFlags"].current & eventFlagEnum["tubeBroken"]) > 0;
    let cacExit = settings.get("cacExit")
        && snes["roomID"].old == roomIDEnum["westCactusAlley"]
        && snes["roomID"].current == roomIDEnum["butterflyRoom"];
    let toilet = settings.get("toilet")
        && (snes["roomID"].old == roomIDEnum["plasmaSpark"]
            && snes["roomID"].current == roomIDEnum["toiletBowl"]
            || snes["roomID"].old == roomIDEnum["oasis"]
                && snes["roomID"].current == roomIDEnum["toiletBowl"]);
    let kronicBoost = settings.get("kronicBoost")
        && (snes["roomID"].old == roomIDEnum["magdolliteTunnel"]
            && snes["roomID"].current == roomIDEnum["kronicBoost"]
            || snes["roomID"].old == roomIDEnum["spikyAcidSnakes"]
                && snes["roomID"].current == roomIDEnum["kronicBoost"]
            || snes["roomID"].old == roomIDEnum["volcano"]
                && snes["roomID"].current == roomIDEnum["kronicBoost"]);
    let lowerNorfairEntrance = settings.get("lowerNorfairEntrance")
        && snes["roomID"].old == roomIDEnum["lowerNorfairElevator"]
        && snes["roomID"].current == roomIDEnum["mainHall"];
    let writg = settings.get("writg")
        && snes["roomID"].old == roomIDEnum["pillars"]
        && snes["roomID"].current == roomIDEnum["writg"];
    let redKiShaft = settings.get("redKiShaft")
        && (snes["roomID"].old == roomIDEnum["amphitheatre"]
            && snes["roomID"].current == roomIDEnum["redKiShaft"]
            || snes["roomID"].old == roomIDEnum["wasteland"]
                && snes["roomID"].current == roomIDEnum["redKiShaft"]);
    let metalPirates = settings.get("metalPirates")
        && snes["roomID"].old == roomIDEnum["wasteland"]
        && snes["roomID"].current == roomIDEnum["metalPirates"];
    let lowerNorfairSpringMaze = settings.get("lowerNorfairSpringMaze")
        && snes["roomID"].old == roomIDEnum["lowerNorfairFireflea"]
        && snes["roomID"].current == roomIDEnum["lowerNorfairSpringMaze"];
    let lowerNorfairExit = settings.get("lowerNorfairExit")
        && snes["roomID"].old == roomIDEnum["threeMusketeers"]
        && snes["roomID"].current == roomIDEnum["singleChamber"];
    let allBossesFinished = (snes["brinstarBosses"].current & bossFlagEnum["kraid"]) > 0
        && (snes["wreckedShipBosses"].current & bossFlagEnum["phantoon"]) > 0
        && (snes["maridiaBosses"].current & bossFlagEnum["draygon"]) > 0
        && (snes["norfairBosses"].current & bossFlagEnum["ridley"]) > 0;
    let goldenFour = settings.get("goldenFour")
        && snes["roomID"].old == roomIDEnum["statuesHallway"]
        && snes["roomID"].current == roomIDEnum["statues"]
        && allBossesFinished;
    let tourianEntrance = settings.get("tourianEntrance")
        && snes["roomID"].old == roomIDEnum["statues"]
        && snes["roomID"].current == roomIDEnum["tourianElevator"];
    let metroids = settings.get("metroids")
        && (snes["roomID"].old == roomIDEnum["metroidOne"]
            && snes["roomID"].current == roomIDEnum["metroidTwo"]
            || snes["roomID"].old == roomIDEnum["metroidTwo"]
                && snes["roomID"].current == roomIDEnum["metroidThree"]
            || snes["roomID"].old == roomIDEnum["metroidThree"]
                && snes["roomID"].current == roomIDEnum["metroidFour"]
            || snes["roomID"].old == roomIDEnum["metroidFour"]
                && snes["roomID"].current == roomIDEnum["tourianHopper"]);
    let babyMetroidRoom = settings.get("babyMetroidRoom")
        && snes["roomID"].old == roomIDEnum["dustTorizo"]
        && snes["roomID"].current == roomIDEnum["bigBoy"];
    let escapeClimb = settings.get("escapeClimb")
        && snes["roomID"].old == roomIDEnum["tourianEscape4"]
        && snes["roomID"].current == roomIDEnum["climb"];
    let roomTransitions = miniBossRooms
        || bossRooms
        || elevatorTransitions
        || ceresEscape
        || wreckedShipEntrance
        || redTowerMiddleEntrance
        || redTowerBottomEntrance
        || kraidsLair
        || risingTideEntrance
        || atticExit
        || tubeBroken
        || cacExit
        || toilet
        || kronicBoost
        || lowerNorfairEntrance
        || writg
        || redKiShaft
        || metalPirates
        || lowerNorfairSpringMaze
        || lowerNorfairExit
        || tourianEntrance
        || goldenFour
        || metroids
        || babyMetroidRoom
        || escapeClimb;

    // Minibosses
    let ceresRidley = settings.get("ceresRidley")
        && (snes["ceresBosses"].old & bossFlagEnum["ceresRidley"]) == 0
        && (snes["ceresBosses"].current & bossFlagEnum["ceresRidley"]) > 0
        && snes["roomID"].current == roomIDEnum["ceresRidley"];
    let bombTorizo = settings.get("bombTorizo")
        && (snes["crateriaBosses"].old & bossFlagEnum["bombTorizo"]) == 0
        && (snes["crateriaBosses"].current & bossFlagEnum["bombTorizo"]) > 0
        && snes["roomID"].current == roomIDEnum["bombTorizo"];
    let sporeSpawn = settings.get("sporeSpawn")
        && (snes["brinstarBosses"].old & bossFlagEnum["sporeSpawn"]) == 0
        && (snes["brinstarBosses"].current & bossFlagEnum["sporeSpawn"]) > 0
        && snes["roomID"].current == roomIDEnum["sporeSpawn"];
    let crocomire = settings.get("crocomire")
        && (snes["norfairBosses"].old & bossFlagEnum["crocomire"]) == 0
        && (snes["norfairBosses"].current & bossFlagEnum["crocomire"]) > 0
        && snes["roomID"].current == roomIDEnum["crocomire"];
    let botwoon = settings.get("botwoon")
        && (snes["maridiaBosses"].old & bossFlagEnum["botwoon"]) == 0
        && (snes["maridiaBosses"].current & bossFlagEnum["botwoon"]) > 0
        && snes["roomID"].current == roomIDEnum["botwoon"];
    let goldenTorizo = settings.get("goldenTorizo")
        && (snes["norfairBosses"].old & bossFlagEnum["goldenTorizo"]) == 0
        && (snes["norfairBosses"].current & bossFlagEnum["goldenTorizo"]) > 0
        && snes["roomID"].current == roomIDEnum["goldenTorizo"];
    let minibossDefeat =
        ceresRidley || bombTorizo || sporeSpawn || crocomire || botwoon || goldenTorizo;

    // Bosses
    let kraid = settings.get("kraid")
        && (snes["brinstarBosses"].old & bossFlagEnum["kraid"]) == 0
        && (snes["brinstarBosses"].current & bossFlagEnum["kraid"]) > 0
        && snes["roomID"].current == roomIDEnum["kraid"];
    if kraid {
        println!("Split due to kraid defeat");
    }
    let phantoon = settings.get("phantoon")
        && (snes["wreckedShipBosses"].old & bossFlagEnum["phantoon"]) == 0
        && (snes["wreckedShipBosses"].current & bossFlagEnum["phantoon"]) > 0
        && snes["roomID"].current == roomIDEnum["phantoon"];
    if phantoon {
        println!("Split due to phantoon defeat");
    }
    let draygon = settings.get("draygon")
        && (snes["maridiaBosses"].old & bossFlagEnum["draygon"]) == 0
        && (snes["maridiaBosses"].current & bossFlagEnum["draygon"]) > 0
        && snes["roomID"].current == roomIDEnum["draygon"];
    if draygon {
        println!("Split due to draygon defeat");
    }
    let ridley = settings.get("ridley")
        && (snes["norfairBosses"].old & bossFlagEnum["ridley"]) == 0
        && (snes["norfairBosses"].current & bossFlagEnum["ridley"]) > 0
        && snes["roomID"].current == roomIDEnum["ridley"];
    if ridley {
        println!("Split due to ridley defeat");
    }
    // Mother Brain phases
    let inMotherBrainRoom = snes["roomID"].current == roomIDEnum["motherBrain"];
    let mb1 = settings.get("mb1")
        && inMotherBrainRoom
        && snes["gameState"].current == gameStateEnum["normalGameplay"]
        && snes["motherBrainHP"].old == 0
        && snes["motherBrainHP"].current == (motherBrainMaxHPEnum["phase2"]);
    if mb1 {
        println!("Split due to mb1 defeat");
    }
    let mb2 = settings.get("mb2")
        && inMotherBrainRoom
        && snes["gameState"].current == gameStateEnum["normalGameplay"]
        && snes["motherBrainHP"].old == 0
        && snes["motherBrainHP"].current == (motherBrainMaxHPEnum["phase3"]);
    if mb2 {
        println!("Split due to mb2 defeat");
    }
    let mb3 = settings.get("mb3")
        && inMotherBrainRoom
        && (snes["tourianBosses"].old & bossFlagEnum["motherBrain"]) == 0
        && (snes["tourianBosses"].current & bossFlagEnum["motherBrain"]) > 0;
    if mb3 {
        println!("Split due to mb3 defeat");
    }
    let bossDefeat = kraid || phantoon || draygon || ridley || mb1 || mb2 || mb3;

    // Run-ending splits
    let escape = settings.get("rtaFinish")
        && (snes["eventFlags"].current & eventFlagEnum["zebesAblaze"]) > 0
        && snes["shipAI"].old != 0xaa4f
        && snes["shipAI"].current == 0xaa4f;

    let takeoff = settings.get("igtFinish")
        && snes["roomID"].current == roomIDEnum["landingSite"]
        && snes["gameState"].old == gameStateEnum["preEndCutscene"]
        && snes["gameState"].current == gameStateEnum["endCutscene"];

    let mut sporeSpawnRTAFinish = false;
    if settings.get("sporeSpawnRTAFinish") {
        if snes.pickedUpSporeSpawnSuper {
            if snes["igtFrames"].old != snes["igtFrames"].current {
                sporeSpawnRTAFinish = true;
                snes.pickedUpSporeSpawnSuper = false;
            }
        } else {
            snes.pickedUpSporeSpawnSuper = snes["roomID"].current == roomIDEnum["sporeSpawnSuper"]
                && (snes["maxSupers"].old + 5) == (snes["maxSupers"].current)
                && (snes["brinstarBosses"].current & bossFlagEnum["sporeSpawn"]) > 0;
        }
    }

    let mut hundredMissileRTAFinish = false;
    if settings.get("hundredMissileRTAFinish") {
        if snes.pickedUpHundredthMissile {
            if snes["igtFrames"].old != snes["igtFrames"].current {
                hundredMissileRTAFinish = true;
                snes.pickedUpHundredthMissile = false;
            }
        } else {
            snes.pickedUpHundredthMissile =
                snes["maxMissiles"].old == 95 && snes["maxMissiles"].current == 100;
        }
    }

    let nonStandardCategoryFinish = sporeSpawnRTAFinish || hundredMissileRTAFinish;

    if pickup {
        println!("Split due to pickup");
    }
    if unlock {
        println!("Split due to unlock");
    }
    if beam {
        println!("Split due to beam upgrade");
    }
    if energyUpgrade {
        println!("Split due to energy upgrade");
    }
    if roomTransitions {
        println!("Split due to room transition");
    }
    if minibossDefeat {
        println!("Split due to miniboss defeat");
    }
    // individual boss defeat conditions already covered above
    if escape {
        println!("Split due to escape");
    }
    if takeoff {
        println!("Split due to takeoff");
    }
    if nonStandardCategoryFinish {
        println!("Split due to non standard category finish");
    }

    pickup
        || unlock
        || beam
        || energyUpgrade
        || roomTransitions
        || minibossDefeat
        || bossDefeat
        || escape
        || takeoff
        || nonStandardCategoryFinish
}

#[derive(Debug, Copy, Clone)]
pub enum Width {
    Byte,
    Word,
}

#[derive(Clone)]
pub struct MemoryWatcher {
    address: u32,
    current: u32,
    old: u32,
    width: Width,
}

impl MemoryWatcher {
    pub fn new(address: u32, width: Width) -> MemoryWatcher {
        MemoryWatcher {
            address,
            current: 0,
            old: 0,
            width,
        }
    }

    fn update_value(&mut self, memory: &[u8]) {
        match self.width {
            Width::Byte => {
                self.old = self.current;
                self.current = memory[self.address as usize] as u32;
            }
            Width::Word => {
                let address = self.address as usize;
                self.old = self.current;
                let word: u16 = (memory[address + 1] as u16) << 8 | memory[address] as u16;
                self.current = word as u32;
            }
        }
    }
}

#[allow(non_snake_case)]
#[derive(Clone)]
pub struct SNESState {
    vars: HashMap<&'static str, MemoryWatcher>,
    pickedUpHundredthMissile: bool,
    pickedUpSporeSpawnSuper: bool,
    latency_samples: VecDeque<u128>,
    data: Vec<u8>,
    // The MemoryWatchers are not in a good
    // state until they've been updated
    // twice, due to having both old and current
    // fields. So the first time we update, we
    // need to do it twice.
    do_extra_update: bool,
}

const NUM_LATENCY_SAMPLES: usize = 10;

impl SNESState {
    pub fn new() -> SNESState {
        let data = vec![0; 0x10000];
        SNESState {
            do_extra_update: true,
            data,
            latency_samples: VecDeque::from([]),
            pickedUpHundredthMissile: false,
            pickedUpSporeSpawnSuper: false,
            vars: HashMap::from([
                // Word
                ("controller", MemoryWatcher::new(0x008B, Width::Word)),
                ("roomID", MemoryWatcher::new(0x079B, Width::Word)),
                ("enemyHP", MemoryWatcher::new(0x0F8C, Width::Word)),
                ("shipAI", MemoryWatcher::new(0x0FB2, Width::Word)),
                ("motherBrainHP", MemoryWatcher::new(0x0FCC, Width::Word)),
                // Byte
                ("mapInUse", MemoryWatcher::new(0x079F, Width::Byte)),
                ("gameState", MemoryWatcher::new(0x0998, Width::Byte)),
                ("unlockedEquips2", MemoryWatcher::new(0x09A4, Width::Byte)),
                ("unlockedEquips", MemoryWatcher::new(0x09A5, Width::Byte)),
                ("unlockedBeams", MemoryWatcher::new(0x09A8, Width::Byte)),
                ("unlockedCharge", MemoryWatcher::new(0x09A9, Width::Byte)),
                ("maxEnergy", MemoryWatcher::new(0x09C4, Width::Word)),
                ("maxMissiles", MemoryWatcher::new(0x09C8, Width::Byte)),
                ("maxSupers", MemoryWatcher::new(0x09CC, Width::Byte)),
                ("maxPowerBombs", MemoryWatcher::new(0x09D0, Width::Byte)),
                ("maxReserve", MemoryWatcher::new(0x09D4, Width::Word)),
                ("igtFrames", MemoryWatcher::new(0x09DA, Width::Byte)),
                ("igtSeconds", MemoryWatcher::new(0x09DC, Width::Byte)),
                ("igtMinutes", MemoryWatcher::new(0x09DE, Width::Byte)),
                ("igtHours", MemoryWatcher::new(0x09E0, Width::Byte)),
                ("playerState", MemoryWatcher::new(0x0A28, Width::Byte)),
                ("eventFlags", MemoryWatcher::new(0xD821, Width::Byte)),
                ("crateriaBosses", MemoryWatcher::new(0xD828, Width::Byte)),
                ("brinstarBosses", MemoryWatcher::new(0xD829, Width::Byte)),
                ("norfairBosses", MemoryWatcher::new(0xD82A, Width::Byte)),
                ("wreckedShipBosses", MemoryWatcher::new(0xD82B, Width::Byte)),
                ("maridiaBosses", MemoryWatcher::new(0xD82C, Width::Byte)),
                ("tourianBosses", MemoryWatcher::new(0xD82D, Width::Byte)),
                ("ceresBosses", MemoryWatcher::new(0xD82E, Width::Byte)),
                ("crateriaItems", MemoryWatcher::new(0xD870, Width::Byte)),
                ("brinteriaItems", MemoryWatcher::new(0xD871, Width::Byte)),
                ("brinstarItems2", MemoryWatcher::new(0xD872, Width::Byte)),
                ("brinstarItems3", MemoryWatcher::new(0xD873, Width::Byte)),
                ("brinstarItems4", MemoryWatcher::new(0xD874, Width::Byte)),
                ("brinstarItems5", MemoryWatcher::new(0xD875, Width::Byte)),
                ("norfairItems1", MemoryWatcher::new(0xD876, Width::Byte)),
                ("norfairItems2", MemoryWatcher::new(0xD877, Width::Byte)),
                ("norfairItems3", MemoryWatcher::new(0xD878, Width::Byte)),
                ("norfairItems4", MemoryWatcher::new(0xD879, Width::Byte)),
                ("norfairItems5", MemoryWatcher::new(0xD87A, Width::Byte)),
                ("wreckedShipItems", MemoryWatcher::new(0xD880, Width::Byte)),
                ("maridiaItems1", MemoryWatcher::new(0xD881, Width::Byte)),
                ("maridiaItems2", MemoryWatcher::new(0xD882, Width::Byte)),
                ("maridiaItems3", MemoryWatcher::new(0xD883, Width::Byte)),
            ]),
        }
    }

    fn update(&mut self) {
        for watcher in self.vars.iter_mut() {
            if self.do_extra_update {
                watcher.1.update_value(&self.data);
                self.do_extra_update = false;
            }
            watcher.1.update_value(&self.data);
        }
    }

    #[expect(clippy::needless_range_loop)]
    pub fn fetch_all(
        &mut self,
        client: &mut crate::usb2snes::SyncClient,
        settings: &Settings,
    ) -> Result<SNESSummary> {
        let start_time = Instant::now();
        let snes_data = client.get_addresses(&[
            (0xF5008B, 2),  // Controller 1 Input
            (0xF5079B, 3),  // ROOM ID + ROOM # for region + Region Number
            (0xF50998, 1),  // GAME STATE
            (0xF509A4, 61), // ITEMS
            (0xF50A28, 1),
            (0xF50F8C, 66),
            (0xF5D821, 14),
            (0xF5D870, 20),
        ])?;
        // TODO: refactor this
        for i in 0..2 {
            self.data[0x008b + i] = snes_data[0][i];
        }
        for i in 0..3 {
            self.data[0x079b + i] = snes_data[1][i];
        }
        self.data[0x0998] = snes_data[2][0];
        for i in 0..61 {
            self.data[0x09a4 + i] = snes_data[3][i];
        }
        self.data[0x0a28] = snes_data[4][0];
        for i in 0..66 {
            self.data[0x0f8c + i] = snes_data[5][i];
        }
        for i in 0..14 {
            self.data[0xd821 + i] = snes_data[6][i];
        }
        for i in 0..20 {
            self.data[0xd870 + i] = snes_data[7][i];
        }
        self.update();
        let start = self.start();
        let reset = self.reset();
        let split = split(settings, self);
        let elapsed = start_time.elapsed().as_millis();
        if self.latency_samples.len() == NUM_LATENCY_SAMPLES {
            self.latency_samples.pop_front();
        }
        self.latency_samples.push_back(elapsed);
        let average_latency: f32 =
            self.latency_samples.iter().sum::<u128>() as f32 / self.latency_samples.len() as f32;
        let mut s = 0;
        for x in self.latency_samples.iter() {
            let y = *x as i128;
            let avg = average_latency as i128;
            let diff = y - avg;
            s += diff * diff;
        }
        let stddev = (s as f32 / (self.latency_samples.len() as f32 - 1.0)).sqrt();
        Ok(SNESSummary {
            latency_average: average_latency,
            latency_stddev: stddev,
            start,
            reset,
            split,
        })
    }

    pub fn start(&self) -> bool {
        let normal_start = self["gameState"].old == 2 && self["gameState"].current == 0x1f;
        // Allow for a cutscene start, even though it's not normally used for speedrunning
        let cutscene_ended = self["gameState"].old == 0x1E && self["gameState"].current == 0x1F;
        // Some categories start from Zebes, such as Spore Spawn RTA
        let zebes_start = self["gameState"].old == 5 && self["gameState"].current == 6;
        normal_start || cutscene_ended || zebes_start
    }

    pub fn reset(&self) -> bool {
        self["roomID"].old != 0 && self["roomID"].current == 0
    }

    pub fn gametime_to_seconds(&self) -> TimeSpan {
        let hours = Duration::hours(self.vars.get("igtHours").unwrap().current.into());

        let minutes = Duration::minutes(self.vars.get("igtMinutes").unwrap().current.into());

        let seconds = Duration::seconds(self.vars.get("igtSeconds").unwrap().current.into());

        let gametime = hours + minutes + seconds;
        TimeSpan::from_seconds(gametime.as_seconds_f64())
    }
}

impl Default for SNESState {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<&str> for SNESState {
    type Output = MemoryWatcher;

    fn index(&self, var: &str) -> &Self::Output {
        self.vars.get(var).unwrap()
    }
}

pub struct SuperMetroidAutoSplitter {
    snes: SNESState,
    settings: Arc<RwLock<Settings>>,
}

impl SuperMetroidAutoSplitter {
    pub fn new(settings: Arc<RwLock<Settings>>) -> Self {
        Self {
            snes: SNESState::new(),
            settings,
        }
    }
}

impl AutoSplitter for SuperMetroidAutoSplitter {
    fn update(
        &mut self,
        client: &mut crate::usb2snes::SyncClient,
    ) -> Result<crate::autosplitters::SNESSummary> {
        self.snes.fetch_all(client, &self.settings.read())
    }

    fn gametime_to_seconds(&self) -> Option<TimeSpan> {
        Some(self.snes.gametime_to_seconds())
    }

    fn reset_game_tracking(&mut self) {
        self.snes = SNESState::new();
    }
}
