#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release mode
#![allow(non_upper_case_globals)]
#[macro_use]
extern crate lazy_static;

use eframe::egui;
use egui::containers::ScrollArea;
//use egui_extras::{Size, TableBuilder};
use livesplit_core::{Run, Segment, Timer};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::ops::Index;
use std::time::Instant;
mod usb2snes;

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
        m.insert("etacoonETankRoom",               0xA011 );
        m.insert("etacoonSuperRoom",               0xA051 );
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

struct Settings {
    data: HashMap<String, (bool, Option<String>)>,
}

impl Settings {
    fn new() -> Self {
        let mut settings = Settings {
            data: HashMap::new(),
        };
        // Split on Missiles, Super Missiles, and Power Bombs
        settings.insert("ammoPickups".to_owned(), true);
        // Split on the first Missile pickup
        settings.insert_with_parent("firstMissile".to_owned(), false, "ammoPickups".to_owned());
        // Split on each Missile upgrade
        settings.insert_with_parent("allMissiles".to_owned(), false, "ammoPickups".to_owned());
        // Split on specific Missile Pack locations
        settings.insert_with_parent(
            "specificMissiles".to_owned(),
            false,
            "ammoPickups".to_owned(),
        );
        // Split on Crateria Missile Pack locations
        settings.insert_with_parent(
            "crateriaMissiles".to_owned(),
            false,
            "specificMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack located at the bottom left of the West Ocean
        settings.insert_with_parent(
            "oceanBottomMissiles".to_owned(),
            false,
            "crateriaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack located in the ceiling tile in West Ocean
        settings.insert_with_parent(
            "oceanTopMissiles".to_owned(),
            false,
            "crateriaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack located in the Morphball maze section of West Ocean
        settings.insert_with_parent(
            "oceanMiddleMissiles".to_owned(),
            false,
            "crateriaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in The Moat, also known as The Lake
        settings.insert_with_parent(
            "moatMissiles".to_owned(),
            false,
            "crateriaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Pit Room
        settings.insert_with_parent(
            "oldTourianMissiles".to_owned(),
            false,
            "crateriaMissiles".to_owned(),
        );
        // Split on picking up the right side Missile Pack at the end of Gauntlet(Green Pirates Shaft)
        settings.insert_with_parent(
            "gauntletRightMissiles".to_owned(),
            false,
            "crateriaMissiles".to_owned(),
        );
        // Split on picking up the left side Missile Pack at the end of Gauntlet(Green Pirates Shaft)
        settings.insert_with_parent(
            "gauntletLeftMissiles".to_owned(),
            false,
            "crateriaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack located in The Final Missile
        settings.insert_with_parent(
            "dentalPlan".to_owned(),
            false,
            "crateriaMissiles".to_owned(),
        );
        // Split on Brinstar Missile Pack locations
        settings.insert_with_parent(
            "brinstarMissiles".to_owned(),
            false,
            "specificMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack located below the crumble bridge in the Early Supers Room
        settings.insert_with_parent(
            "earlySuperBridgeMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the first Missile Pack behind the Brinstar Reserve Tank
        settings.insert_with_parent(
            "greenBrinstarReserveMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the second Missile Pack behind the Brinstar Reserve Tank Room
        settings.insert_with_parent(
            "greenBrinstarExtraReserveMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack located left of center in Big Pink
        settings.insert_with_parent(
            "bigPinkTopMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack located at the bottom left of Big Pink
        settings.insert_with_parent(
            "chargeMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Green Hill Zone
        settings.insert_with_parent(
            "greenHillsMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Blue Brinstar Energy Tank Room
        settings.insert_with_parent(
            "blueBrinstarETankMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the first Missile Pack of the game(First Missile Room)
        settings.insert_with_parent(
            "alphaMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack located on the pedestal in Billy Mays' Room
        settings.insert_with_parent(
            "billyMaysMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack located in the floor of Billy Mays' Room
        settings.insert_with_parent(
            "butWaitTheresMoreMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Alpha Power Bombs Room
        settings.insert_with_parent(
            "redBrinstarMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Warehouse Kihunter Room
        settings.insert_with_parent(
            "warehouseMissiles".to_owned(),
            false,
            "brinstarMissiles".to_owned(),
        );
        // Split on Norfair Missile Pack locations
        settings.insert_with_parent(
            "norfairMissiles".to_owned(),
            false,
            "specificMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Cathedral
        settings.insert_with_parent(
            "cathedralMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Crumble Shaft
        settings.insert_with_parent(
            "crumbleShaftMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Crocomire Escape
        settings.insert_with_parent(
            "crocomireEscapeMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Hi Jump Energy Tank Room
        settings.insert_with_parent(
            "hiJumpMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Post Crocomire Missile Room, also known as Cosine Room
        settings.insert_with_parent(
            "postCrocomireMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Post Crocomire Jump Room
        settings.insert_with_parent(
            "grappleMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Norfair Reserve Tank Room
        settings.insert_with_parent(
            "norfairReserveMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Green Bubbles Missile Room
        settings.insert_with_parent(
            "greenBubblesMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Bubble Mountain
        settings.insert_with_parent(
            "bubbleMountainMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Speed Booster Hall
        settings.insert_with_parent(
            "speedBoostMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Wave Missile Pack in Double Chamber
        settings.insert_with_parent(
            "waveMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Golden Torizo's Room
        settings.insert_with_parent(
            "goldTorizoMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Mickey Mouse Room
        settings.insert_with_parent(
            "mickeyMouseMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Lower Norfair Springball Maze Room
        settings.insert_with_parent(
            "lowerNorfairSpringMazeMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the The Musketeers' Room
        settings.insert_with_parent(
            "threeMusketeersMissiles".to_owned(),
            false,
            "norfairMissiles".to_owned(),
        );
        // Split on Wrecked Ship Missile Pack locations
        settings.insert_with_parent(
            "wreckedShipMissiles".to_owned(),
            false,
            "specificMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Wrecked Ship Main Shaft
        settings.insert_with_parent(
            "wreckedShipMainShaftMissiles".to_owned(),
            false,
            "wreckedShipMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Bowling Alley
        settings.insert_with_parent(
            "bowlingMissiles".to_owned(),
            false,
            "wreckedShipMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Wrecked Ship East Missile Room
        settings.insert_with_parent(
            "atticMissiles".to_owned(),
            false,
            "wreckedShipMissiles".to_owned(),
        );
        // Split on Maridia Missile Pack locations
        settings.insert_with_parent(
            "maridiaMissiles".to_owned(),
            false,
            "specificMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Main Street
        settings.insert_with_parent(
            "mainStreetMissiles".to_owned(),
            false,
            "maridiaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Mama Turtle Room
        settings.insert_with_parent(
            "mamaTurtleMissiles".to_owned(),
            false,
            "maridiaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Watering Hole
        settings.insert_with_parent(
            "wateringHoleMissiles".to_owned(),
            false,
            "maridiaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in the Pseudo Plasma Spark Room
        settings.insert_with_parent(
            "beachMissiles".to_owned(),
            false,
            "maridiaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in West Sand Hole
        settings.insert_with_parent(
            "leftSandPitMissiles".to_owned(),
            false,
            "maridiaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in East Sand Hole
        settings.insert_with_parent(
            "rightSandPitMissiles".to_owned(),
            false,
            "maridiaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in Aqueduct
        settings.insert_with_parent(
            "aqueductMissiles".to_owned(),
            false,
            "maridiaMissiles".to_owned(),
        );
        // Split on picking up the Missile Pack in The Precious Room
        settings.insert_with_parent(
            "preDraygonMissiles".to_owned(),
            false,
            "maridiaMissiles".to_owned(),
        );
        // Split on the first Super Missile pickup
        settings.insert_with_parent("firstSuper".to_owned(), false, "ammoPickups".to_owned());
        // Split on each Super Missile upgrade
        settings.insert_with_parent("allSupers".to_owned(), false, "ammoPickups".to_owned());
        // Split on specific Super Missile Pack locations
        settings.insert_with_parent("specificSupers".to_owned(), false, "ammoPickups".to_owned());
        // Split on picking up the Super Missile Pack in the Crateria Super Room
        settings.insert_with_parent("climbSupers".to_owned(), false, "specificSupers".to_owned());
        // Split on picking up the Super Missile Pack in the Spore Spawn Super Room (NOTE: SSTRA splits when the dialogue box disappears, not on touch. Use Spore Spawn RTA Finish for SSTRA runs.)
        settings.insert_with_parent(
            "sporeSpawnSupers".to_owned(),
            false,
            "specificSupers".to_owned(),
        );
        // Split on picking up the Super Missile Pack in the Early Supers Room
        settings.insert_with_parent("earlySupers".to_owned(), false, "specificSupers".to_owned());
        // Split on picking up the Super Missile Pack in the Etacoon Super Room
        settings.insert_with_parent(
            "etacoonSupers".to_owned(),
            false,
            "specificSupers".to_owned(),
        );
        // Split on picking up the Super Missile Pack in the Golden Torizo's Room
        settings.insert_with_parent(
            "goldTorizoSupers".to_owned(),
            false,
            "specificSupers".to_owned(),
        );
        // Split on picking up the Super Missile Pack in the Wrecked Ship West Super Room
        settings.insert_with_parent(
            "wreckedShipLeftSupers".to_owned(),
            false,
            "specificSupers".to_owned(),
        );
        // Split on picking up the Super Missile Pack in the Wrecked Ship East Super Room
        settings.insert_with_parent(
            "wreckedShipRightSupers".to_owned(),
            false,
            "specificSupers".to_owned(),
        );
        // Split on picking up the Super Missile Pack in Main Street
        settings.insert_with_parent("crabSupers".to_owned(), false, "specificSupers".to_owned());
        // Split on picking up the Super Missile Pack in Watering Hole
        settings.insert_with_parent(
            "wateringHoleSupers".to_owned(),
            false,
            "specificSupers".to_owned(),
        );
        // Split on picking up the Super Missile Pack in Aqueduct
        settings.insert_with_parent(
            "aqueductSupers".to_owned(),
            false,
            "specificSupers".to_owned(),
        );
        // Split on the first Power Bomb pickup
        settings.insert_with_parent("firstPowerBomb".to_owned(), true, "ammoPickups".to_owned());
        // Split on each Power Bomb upgrade
        settings.insert_with_parent("allPowerBombs".to_owned(), false, "ammoPickups".to_owned());
        // Split on specific Power Bomb Pack locations
        settings.insert_with_parent("specificBombs".to_owned(), false, "ammoPickups".to_owned());
        // Split on picking up the Power Bomb Pack in the Crateria Power Bomb Room
        settings.insert_with_parent(
            "landingSiteBombs".to_owned(),
            false,
            "specificBombs".to_owned(),
        );
        // Split on picking up the Power Bomb Pack in the Etacoon Room section of Green Brinstar Main Shaft
        settings.insert_with_parent("etacoonBombs".to_owned(), false, "specificBombs".to_owned());
        // Split on picking up the Power Bomb Pack in the Pink Brinstar Power Bomb Room
        settings.insert_with_parent(
            "pinkBrinstarBombs".to_owned(),
            false,
            "specificBombs".to_owned(),
        );
        // Split on picking up the Power Bomb Pack in the Morph Ball Room
        settings.insert_with_parent(
            "blueBrinstarBombs".to_owned(),
            false,
            "specificBombs".to_owned(),
        );
        // Split on picking up the Power Bomb Pack in the Alpha Power Bomb Room
        settings.insert_with_parent("alphaBombs".to_owned(), false, "specificBombs".to_owned());
        // Split on picking up the Power Bomb Pack in the Beta Power Bomb Room
        settings.insert_with_parent("betaBombs".to_owned(), false, "specificBombs".to_owned());
        // Split on picking up the Power Bomb Pack in the Post Crocomire Power Bomb Room
        settings.insert_with_parent(
            "crocomireBombs".to_owned(),
            false,
            "specificBombs".to_owned(),
        );
        // Split on picking up the Power Bomb Pack in the Lower Norfair Escape Power Bomb Room
        settings.insert_with_parent(
            "lowerNorfairEscapeBombs".to_owned(),
            false,
            "specificBombs".to_owned(),
        );
        // Split on picking up the Power Bomb Pack in Wasteland
        settings.insert_with_parent("shameBombs".to_owned(), false, "specificBombs".to_owned());
        // Split on picking up the Power Bomb Pack in East Sand Hall
        settings.insert_with_parent(
            "rightSandPitBombs".to_owned(),
            false,
            "specificBombs".to_owned(),
        );

        // Split on Varia and Gravity pickups
        settings.insert("suitUpgrades".to_owned(), true);
        // Split on picking up the Varia Suit
        settings.insert_with_parent("variaSuit".to_owned(), true, "suitUpgrades".to_owned());
        // Split on picking up the Gravity Suit
        settings.insert_with_parent("gravSuit".to_owned(), true, "suitUpgrades".to_owned());

        // Split on beam upgrades
        settings.insert("beamUpgrades".to_owned(), true);
        // Split on picking up the Charge Beam
        settings.insert_with_parent("chargeBeam".to_owned(), false, "beamUpgrades".to_owned());
        // Split on picking up the Spazer
        settings.insert_with_parent("spazer".to_owned(), false, "beamUpgrades".to_owned());
        // Split on picking up the Wave Beam
        settings.insert_with_parent("wave".to_owned(), true, "beamUpgrades".to_owned());
        // Split on picking up the Ice Beam
        settings.insert_with_parent("ice".to_owned(), false, "beamUpgrades".to_owned());
        // Split on picking up the Plasma Beam
        settings.insert_with_parent("plasma".to_owned(), false, "beamUpgrades".to_owned());

        // Split on boot upgrades
        settings.insert("bootUpgrades".to_owned(), false);
        // Split on picking up the Hi-Jump Boots
        settings.insert_with_parent("hiJump".to_owned(), false, "bootUpgrades".to_owned());
        // Split on picking up Space Jump
        settings.insert_with_parent("spaceJump".to_owned(), false, "bootUpgrades".to_owned());
        // Split on picking up the Speed Booster
        settings.insert_with_parent("speedBooster".to_owned(), false, "bootUpgrades".to_owned());

        // Split on Energy Tanks and Reserve Tanks
        settings.insert("energyUpgrades".to_owned(), false);
        // Split on picking up the first Energy Tank
        settings.insert_with_parent("firstETank".to_owned(), false, "energyUpgrades".to_owned());
        // Split on picking up each Energy Tank
        settings.insert_with_parent("allETanks".to_owned(), false, "energyUpgrades".to_owned());
        // Split on specific Energy Tank locations
        settings.insert_with_parent(
            "specificETanks".to_owned(),
            false,
            "energyUpgrades".to_owned(),
        );
        // Split on picking up the Energy Tank in the Gauntlet Energy Tank Room
        settings.insert_with_parent(
            "gauntletETank".to_owned(),
            false,
            "specificETanks".to_owned(),
        );
        // Split on picking up the Energy Tank in the Terminator Room
        settings.insert_with_parent(
            "terminatorETank".to_owned(),
            false,
            "specificETanks".to_owned(),
        );
        // Split on picking up the Energy Tank in the Blue Brinstar Energy Tank Room
        settings.insert_with_parent(
            "ceilingETank".to_owned(),
            false,
            "specificETanks".to_owned(),
        );
        // Split on picking up the Energy Tank in the Etacoon Energy Tank Room
        settings.insert_with_parent(
            "etecoonsETank".to_owned(),
            false,
            "specificETanks".to_owned(),
        );
        // Split on picking up the Energy Tank in Waterway
        settings.insert_with_parent(
            "waterwayETank".to_owned(),
            false,
            "specificETanks".to_owned(),
        );
        // Split on picking up the Energy Tank in the Hopper Energy Tank Room
        settings.insert_with_parent(
            "waveGateETank".to_owned(),
            false,
            "specificETanks".to_owned(),
        );
        // Split on picking up the Kraid Energy Tank in the Warehouse Energy Tank Room
        settings.insert_with_parent("kraidETank".to_owned(), false, "specificETanks".to_owned());
        // Split on picking up the Energy Tank in Crocomire's Room
        settings.insert_with_parent(
            "crocomireETank".to_owned(),
            false,
            "specificETanks".to_owned(),
        );
        // Split on picking up the Energy Tank in the Hi Jump Energy Tank Room
        settings.insert_with_parent("hiJumpETank".to_owned(), false, "specificETanks".to_owned());
        // Split on picking up the Energy Tank in the Ridley Tank Room
        settings.insert_with_parent("ridleyETank".to_owned(), false, "specificETanks".to_owned());
        // Split on picking up the Energy Tank in the Lower Norfair Fireflea Room
        settings.insert_with_parent(
            "firefleaETank".to_owned(),
            false,
            "specificETanks".to_owned(),
        );
        // Split on picking up the Energy Tank in the Wrecked Ship Energy Tank Room
        settings.insert_with_parent(
            "wreckedShipETank".to_owned(),
            false,
            "specificETanks".to_owned(),
        );
        // Split on picking up the Energy Tank in the Mama Turtle Room
        settings.insert_with_parent("tatoriETank".to_owned(), false, "specificETanks".to_owned());
        // Split on picking up the Energy Tank in the Botwoon Energy Tank Room
        settings.insert_with_parent(
            "botwoonETank".to_owned(),
            false,
            "specificETanks".to_owned(),
        );
        // Split on picking up each Reserve Tank
        settings.insert_with_parent(
            "reserveTanks".to_owned(),
            false,
            "energyUpgrades".to_owned(),
        );
        // Split on specific Reserve Tank locations
        settings.insert_with_parent(
            "specificRTanks".to_owned(),
            false,
            "energyUpgrades".to_owned(),
        );
        // Split on picking up the Reserve Tank in the Brinstar Reserve Tank Room
        settings.insert_with_parent(
            "brinstarReserve".to_owned(),
            false,
            "specificRTanks".to_owned(),
        );
        // Split on picking up the Reserve Tank in the Norfair Reserve Tank Room
        settings.insert_with_parent(
            "norfairReserve".to_owned(),
            false,
            "specificRTanks".to_owned(),
        );
        // Split on picking up the Reserve Tank in Bowling Alley
        settings.insert_with_parent(
            "wreckedShipReserve".to_owned(),
            false,
            "specificRTanks".to_owned(),
        );
        // Split on picking up the Reserve Tank in West Sand Hole
        settings.insert_with_parent(
            "maridiaReserve".to_owned(),
            false,
            "specificRTanks".to_owned(),
        );

        // Split on the miscellaneous upgrades
        settings.insert("miscUpgrades".to_owned(), false);
        // Split on picking up the Morphing Ball
        settings.insert_with_parent("morphBall".to_owned(), false, "miscUpgrades".to_owned());
        // Split on picking up the Bomb
        settings.insert_with_parent("bomb".to_owned(), false, "miscUpgrades".to_owned());
        // Split on picking up the Spring Ball
        settings.insert_with_parent("springBall".to_owned(), false, "miscUpgrades".to_owned());
        // Split on picking up the Screw Attack
        settings.insert_with_parent("screwAttack".to_owned(), false, "miscUpgrades".to_owned());
        // Split on picking up the Grapple Beam
        settings.insert_with_parent("grapple".to_owned(), false, "miscUpgrades".to_owned());
        // Split on picking up the X-Ray Scope
        settings.insert_with_parent("xray".to_owned(), false, "miscUpgrades".to_owned());

        // Split on transitions between areas
        settings.insert("areaTransitions".to_owned(), true);
        // Split on entering miniboss rooms (except Bomb Torizo)
        settings.insert_with_parent(
            "miniBossRooms".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on entering major boss rooms
        settings.insert_with_parent("bossRooms".to_owned(), false, "areaTransitions".to_owned());
        // Split on elevator transitions between areas (except Statue Room to Tourian)
        settings.insert_with_parent(
            "elevatorTransitions".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on leaving Ceres Station
        settings.insert_with_parent(
            "ceresEscape".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on entering the Wrecked Ship Entrance from the lower door of West Ocean
        settings.insert_with_parent(
            "wreckedShipEntrance".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on entering Red Tower from Noob Bridge
        settings.insert_with_parent(
            "redTowerMiddleEntrance".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on entering Red Tower from Skree Boost room
        settings.insert_with_parent(
            "redTowerBottomEntrance".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on entering Kraid's Lair
        settings.insert_with_parent("kraidsLair".to_owned(), false, "areaTransitions".to_owned());
        // Split on entering Rising Tide from Cathedral
        settings.insert_with_parent(
            "risingTideEntrance".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on exiting Attic
        settings.insert_with_parent("atticExit".to_owned(), false, "areaTransitions".to_owned());
        // Split on blowing up the tube to enter Maridia
        settings.insert_with_parent("tubeBroken".to_owned(), false, "areaTransitions".to_owned());
        // Split on exiting West Cacattack Alley
        settings.insert_with_parent("cacExit".to_owned(), false, "areaTransitions".to_owned());
        // Split on entering Toilet Bowl from either direction
        settings.insert_with_parent("toilet".to_owned(), false, "areaTransitions".to_owned());
        // Split on entering Kronic Boost room
        settings.insert_with_parent(
            "kronicBoost".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on the elevator down to Lower Norfair
        settings.insert_with_parent(
            "lowerNorfairEntrance".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on entering Worst Room in the Game
        settings.insert_with_parent("writg".to_owned(), false, "areaTransitions".to_owned());
        // Split on entering Red Kihunter Shaft from either Amphitheatre or Wastelands (NOTE: will split twice)
        settings.insert_with_parent("redKiShaft".to_owned(), false, "areaTransitions".to_owned());
        // Split on entering Metal Pirates Room from Wasteland
        settings.insert_with_parent(
            "metalPirates".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on entering Lower Norfair Springball Maze Room
        settings.insert_with_parent(
            "lowerNorfairSpringMaze".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on moving from the Three Musketeers' Room to the Single Chamber
        settings.insert_with_parent(
            "lowerNorfairExit".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on entering the Statues Room with all four major bosses defeated
        settings.insert_with_parent("goldenFour".to_owned(), true, "areaTransitions".to_owned());
        // Split on the elevator down to Tourian
        settings.insert_with_parent(
            "tourianEntrance".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on exiting each of the Metroid rooms in Tourian
        settings.insert_with_parent("metroids".to_owned(), false, "areaTransitions".to_owned());
        // Split on moving from the Dust Torizo Room to the Big Boy Room
        settings.insert_with_parent(
            "babyMetroidRoom".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );
        // Split on moving from Tourian Escape Room 4 to The Climb
        settings.insert_with_parent(
            "escapeClimb".to_owned(),
            false,
            "areaTransitions".to_owned(),
        );

        // Split on defeating minibosses
        settings.insert("miniBosses".to_owned(), false);
        // Split on starting the Ceres Escape
        settings.insert_with_parent("ceresRidley".to_owned(), false, "miniBosses".to_owned());
        // Split on Bomb Torizo's drops appearing
        settings.insert_with_parent("bombTorizo".to_owned(), false, "miniBosses".to_owned());
        // Split on the last hit to Spore Spawn
        settings.insert_with_parent("sporeSpawn".to_owned(), false, "miniBosses".to_owned());
        // Split on Crocomire's drops appearing
        settings.insert_with_parent("crocomire".to_owned(), false, "miniBosses".to_owned());
        // Split on Botwoon's vertical column being fully destroyed
        settings.insert_with_parent("botwoon".to_owned(), false, "miniBosses".to_owned());
        // Split on Golden Torizo's drops appearing
        settings.insert_with_parent("goldenTorizo".to_owned(), false, "miniBosses".to_owned());

        // Split on defeating major bosses
        settings.insert("bosses".to_owned(), true);
        // Split shortly after Kraid's drops appear
        settings.insert_with_parent("kraid".to_owned(), false, "bosses".to_owned());
        // Split on Phantoon's drops appearing
        settings.insert_with_parent("phantoon".to_owned(), false, "bosses".to_owned());
        // Split on Draygon's drops appearing
        settings.insert_with_parent("draygon".to_owned(), false, "bosses".to_owned());
        // Split on Ridley's drops appearing
        settings.insert_with_parent("ridley".to_owned(), true, "bosses".to_owned());
        // Split on Mother Brain's head hitting the ground at the end of the first phase
        settings.insert_with_parent("mb1".to_owned(), false, "bosses".to_owned());
        // Split on the Baby Metroid detaching from Mother Brain's head
        settings.insert_with_parent("mb2".to_owned(), true, "bosses".to_owned());
        // Split on the start of the Zebes Escape
        settings.insert_with_parent("mb3".to_owned(), false, "bosses".to_owned());

        // Split on facing forward at the end of Zebes Escape
        settings.insert("rtaFinish".to_owned(), true);
        // Split on In-Game Time finalizing, when the end cutscene starts
        settings.insert("igtFinish".to_owned(), false);
        // Split on the end of a Spore Spawn RTA run, when the text box clears after collecting the Super Missiles
        settings.insert("sporeSpawnRTAFinish".to_owned(), false);
        // Split on the end of a 100 Missile RTA run, when the text box clears after collecting the hundredth missile
        settings.insert("hundredMissileRTAFinish".to_owned(), false);
        settings
    }

    fn insert(&mut self, name: String, value: bool) {
        self.data.insert(name, (value, None));
    }

    fn insert_with_parent(&mut self, name: String, value: bool, parent: String) {
        self.data.insert(name, (value, Some(parent)));
    }

    fn get(&self, var: &str) -> bool {
        match self.data[var] {
            (b, None) => b,
            (b, Some(ref p)) => b && self.get(&p),
        }
    }

    fn set(&mut self, var: String, value: bool) {
        let val = match self.data.get_mut(&var) {
            None => (value, None),
            Some((_, x)) => (value, x.clone()),
        };
        self.data.insert(var, val);
    }

    fn split_on_misc_upgrades(&mut self) {
        self.set("miscUpgrades".to_owned(), true);
        self.set("morphBall".to_owned(), true);
        self.set("bomb".to_owned(), true);
        self.set("springBall".to_owned(), true);
        self.set("screwAttack".to_owned(), true);
        self.set("grapple".to_owned(), true);
        self.set("xray".to_owned(), true);
    }

    fn split_on_hundo(&mut self) {
        self.set("ammoPickups".to_owned(), true);
        self.set("allMissiles".to_owned(), true);
        self.set("allSupers".to_owned(), true);
        self.set("allPowerBombs".to_owned(), true);
        self.set("beamUpgrades".to_owned(), true);
        self.set("chargeBeam".to_owned(), true);
        self.set("spazer".to_owned(), true);
        self.set("wave".to_owned(), true);
        self.set("ice".to_owned(), true);
        self.set("plasma".to_owned(), true);
        self.set("bootUpgrades".to_owned(), true);
        self.set("hiJump".to_owned(), true);
        self.set("spaceJump".to_owned(), true);
        self.set("speedBooster".to_owned(), true);
        self.set("energyUpgrades".to_owned(), true);
        self.set("allETanks".to_owned(), true);
        self.set("reserveTanks".to_owned(), true);
        self.split_on_misc_upgrades();
        self.set("areaTransitions".to_owned(), true); // should already be true
        self.set("tubeBroken".to_owned(), true);
        self.set("ceresEscape".to_owned(), true);
        self.set("bosses".to_owned(), true); // should already be true
        self.set("kraid".to_owned(), true);
        self.set("phantoon".to_owned(), true);
        self.set("draygon".to_owned(), true);
        self.set("ridley".to_owned(), true);
        self.set("mb1".to_owned(), true);
        self.set("mb2".to_owned(), true);
        self.set("mb3".to_owned(), true);
        self.set("miniBosses".to_owned(), true);
        self.set("ceresRidley".to_owned(), true);
        self.set("bombTorizo".to_owned(), true);
        self.set("crocomire".to_owned(), true);
        self.set("botwoon".to_owned(), true);
        self.set("goldenTorizo".to_owned(), true);
        self.set("babyMetroidRoom".to_owned(), true);
    }
    fn split_on_anypercent(&mut self) {
        self.set("ammoPickups".to_owned(), true);
        self.set("specificMissiles".to_owned(), true);
        self.set("specificSupers".to_owned(), true);
        self.set("wreckedShipLeftSupers".to_owned(), true);
        self.set("specificPowerBombs".to_owned(), true);
        self.set("firstMissile".to_owned(), true);
        self.set("firstSuper".to_owned(), true);
        self.set("firstPowerBomb".to_owned(), true);
        self.set("brinstarMissiles".to_owned(), true);
        self.set("norfairMissiles".to_owned(), true);
        self.set("chargeMissiles".to_owned(), true);
        self.set("waveMissiles".to_owned(), true);
        self.set("beamUpgrades".to_owned(), true);
        self.set("chargeBeam".to_owned(), true);
        self.set("wave".to_owned(), true);
        self.set("ice".to_owned(), true);
        self.set("plasma".to_owned(), true);
        self.set("bootUpgrades".to_owned(), true);
        self.set("hiJump".to_owned(), true);
        self.set("speedBooster".to_owned(), true);
        self.set("specificETanks".to_owned(), true);
        self.set("energyUpgrades".to_owned(), true);
        self.set("terminatorETank".to_owned(), true);
        self.set("hiJumpETank".to_owned(), true);
        self.set("botwoonETank".to_owned(), true);
        self.set("miscUpgrades".to_owned(), true);
        self.set("morphBall".to_owned(), true);
        self.set("spaceJump".to_owned(), true);
        self.set("bomb".to_owned(), true);
        self.set("areaTransitions".to_owned(), true); // should already be true
        self.set("tubeBroken".to_owned(), true);
        self.set("ceresEscape".to_owned(), true);
        self.set("bosses".to_owned(), true); // should already be true
        self.set("kraid".to_owned(), true);
        self.set("phantoon".to_owned(), true);
        self.set("draygon".to_owned(), true);
        self.set("ridley".to_owned(), true);
        self.set("mb1".to_owned(), true);
        self.set("mb2".to_owned(), true);
        self.set("mb3".to_owned(), true);
        self.set("miniBosses".to_owned(), true);
        self.set("ceresRidley".to_owned(), true);
        self.set("bombTorizo".to_owned(), true);
        self.set("botwoon".to_owned(), true);
        self.set("goldenTorizo".to_owned(), true);
        self.set("babyMetroidRoom".to_owned(), true);
    }
}

#[allow(non_snake_case)]
struct MyApp {
    client: usb2snes::usb2snes::SyncClient,
    snes: SNESState,
    settings: Settings,
    pickedUpSporeSpawnSuper: bool,
    pickedUpHundredthMissile: bool,
    timer: Timer,
    latency_samples: VecDeque<u128>,
    remaining_space: egui::Vec2,
}

enum Width {
    Byte,
    Word,
}

struct MemoryWatcher {
    address: u32,
    current: u32,
    old: u32,
    width: Width,
}

impl MemoryWatcher {
    fn new(address: u32, width: Width) -> MemoryWatcher {
        MemoryWatcher {
            address: address,
            current: 0,
            old: 0,
            width: width,
        }
    }
}

impl MemoryWatcher {
    fn update_value(&mut self, memory: &Vec<u8>) {
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

struct SNESState {
    vars: HashMap<String, MemoryWatcher>,
}

impl SNESState {
    fn new() -> SNESState {
        SNESState {
            vars: HashMap::from([
                // Word
                (
                    "controller".to_owned(),
                    MemoryWatcher::new(0x008B, Width::Word),
                ),
                ("roomID".to_owned(), MemoryWatcher::new(0x079B, Width::Word)),
                (
                    "enemyHP".to_owned(),
                    MemoryWatcher::new(0x0F8C, Width::Word),
                ),
                ("shipAI".to_owned(), MemoryWatcher::new(0x0FB2, Width::Word)),
                (
                    "motherBrainHP".to_owned(),
                    MemoryWatcher::new(0x0FCC, Width::Word),
                ),
                // Byte
                (
                    "mapInUse".to_owned(),
                    MemoryWatcher::new(0x079F, Width::Byte),
                ),
                (
                    "gameState".to_owned(),
                    MemoryWatcher::new(0x0998, Width::Byte),
                ),
                (
                    "unlockedEquips2".to_owned(),
                    MemoryWatcher::new(0x09A4, Width::Byte),
                ),
                (
                    "unlockedEquips".to_owned(),
                    MemoryWatcher::new(0x09A5, Width::Byte),
                ),
                (
                    "unlockedBeams".to_owned(),
                    MemoryWatcher::new(0x09A8, Width::Byte),
                ),
                (
                    "unlockedCharge".to_owned(),
                    MemoryWatcher::new(0x09A9, Width::Byte),
                ),
                (
                    "maxEnergy".to_owned(),
                    MemoryWatcher::new(0x09C4, Width::Word),
                ),
                (
                    "maxMissiles".to_owned(),
                    MemoryWatcher::new(0x09C8, Width::Byte),
                ),
                (
                    "maxSupers".to_owned(),
                    MemoryWatcher::new(0x09CC, Width::Byte),
                ),
                (
                    "maxPowerBombs".to_owned(),
                    MemoryWatcher::new(0x09D0, Width::Byte),
                ),
                (
                    "maxReserve".to_owned(),
                    MemoryWatcher::new(0x09D4, Width::Word),
                ),
                (
                    "igtFrames".to_owned(),
                    MemoryWatcher::new(0x09DA, Width::Byte),
                ),
                (
                    "igtSeconds".to_owned(),
                    MemoryWatcher::new(0x09DC, Width::Byte),
                ),
                (
                    "igtMinutes".to_owned(),
                    MemoryWatcher::new(0x09DE, Width::Byte),
                ),
                (
                    "igtHours".to_owned(),
                    MemoryWatcher::new(0x09E0, Width::Byte),
                ),
                (
                    "playerState".to_owned(),
                    MemoryWatcher::new(0x0A28, Width::Byte),
                ),
                (
                    "eventFlags".to_owned(),
                    MemoryWatcher::new(0xD821, Width::Byte),
                ),
                (
                    "crateriaBosses".to_owned(),
                    MemoryWatcher::new(0xD828, Width::Byte),
                ),
                (
                    "brinstarBosses".to_owned(),
                    MemoryWatcher::new(0xD829, Width::Byte),
                ),
                (
                    "norfairBosses".to_owned(),
                    MemoryWatcher::new(0xD82A, Width::Byte),
                ),
                (
                    "wreckedShipBosses".to_owned(),
                    MemoryWatcher::new(0xD82B, Width::Byte),
                ),
                (
                    "maridiaBosses".to_owned(),
                    MemoryWatcher::new(0xD82C, Width::Byte),
                ),
                (
                    "tourianBosses".to_owned(),
                    MemoryWatcher::new(0xD82D, Width::Byte),
                ),
                (
                    "ceresBosses".to_owned(),
                    MemoryWatcher::new(0xD82E, Width::Byte),
                ),
                (
                    "crateriaItems".to_owned(),
                    MemoryWatcher::new(0xD870, Width::Byte),
                ),
                (
                    "brinteriaItems".to_owned(),
                    MemoryWatcher::new(0xD871, Width::Byte),
                ),
                (
                    "brinstarItems2".to_owned(),
                    MemoryWatcher::new(0xD872, Width::Byte),
                ),
                (
                    "brinstarItems3".to_owned(),
                    MemoryWatcher::new(0xD873, Width::Byte),
                ),
                (
                    "brinstarItems4".to_owned(),
                    MemoryWatcher::new(0xD874, Width::Byte),
                ),
                (
                    "brinstarItems5".to_owned(),
                    MemoryWatcher::new(0xD875, Width::Byte),
                ),
                (
                    "norfairItems1".to_owned(),
                    MemoryWatcher::new(0xD876, Width::Byte),
                ),
                (
                    "norfairItems2".to_owned(),
                    MemoryWatcher::new(0xD877, Width::Byte),
                ),
                (
                    "norfairItems3".to_owned(),
                    MemoryWatcher::new(0xD878, Width::Byte),
                ),
                (
                    "norfairItems4".to_owned(),
                    MemoryWatcher::new(0xD879, Width::Byte),
                ),
                (
                    "norfairItems5".to_owned(),
                    MemoryWatcher::new(0xD87A, Width::Byte),
                ),
                (
                    "wreckedShipItems".to_owned(),
                    MemoryWatcher::new(0xD880, Width::Byte),
                ),
                (
                    "maridiaItems1".to_owned(),
                    MemoryWatcher::new(0xD881, Width::Byte),
                ),
                (
                    "maridiaItems2".to_owned(),
                    MemoryWatcher::new(0xD882, Width::Byte),
                ),
                (
                    "maridiaItems3".to_owned(),
                    MemoryWatcher::new(0xD883, Width::Byte),
                ),
            ]),
        }
    }

    fn update(&mut self, memory: &Vec<u8>) {
        for watcher in self.vars.iter_mut() {
            watcher.1.update_value(memory);
        }
    }

    fn fetch_all(
        &mut self,
        client: &mut usb2snes::usb2snes::SyncClient,
    ) -> Result<(), Box<dyn Error>> {
        let mut data = Vec::with_capacity(0x10000);
        data.resize(0x10000, 0);
        let snes_data = client.get_addresses(&vec![
            (0xf5008B, 2),
            (0xf5079B, 3),
            (0xf50998, 1),
            (0xf509A4, 61),
            (0xf50A28, 1),
            (0xf50F8C, 66),
            (0xf5D821, 14),
            (0xf5D870, 20),
        ])?;
        // TODO: refactor this
        for i in 0..2 {
            data[0x008b + i] = snes_data[0][i];
        }
        for i in 0..3 {
            data[0x079b + i] = snes_data[1][i];
        }
        data[0x0998] = snes_data[2][0];
        for i in 0..61 {
            data[0x09a4 + i] = snes_data[3][i];
        }
        data[0x0a28] = snes_data[4][0];
        for i in 0..66 {
            data[0x0f8c + i] = snes_data[5][i];
        }
        for i in 0..14 {
            data[0xd821 + i] = snes_data[6][i];
        }
        for i in 0..20 {
            data[0xd870 + i] = snes_data[7][i];
        }
        self.update(&data);
        Ok(())
    }

    fn start(&self) -> bool {
        let normal_start = self["gameState"].old == 2 && self["gameState"].current == 0x1f;
        // Allow for a cutscene start, even though it's not normally used for speedrunning
        let cutscene_ended = self["gameState"].old == 0x1E && self["gameState"].current == 0x1F;
        // Some categories start from Zebes, such as Spore Spawn RTA
        let zebes_start = self["gameState"].old == 5 && self["gameState"].current == 6;
        normal_start || cutscene_ended || zebes_start
    }

    fn reset(&self) -> bool {
        self["roomID"].old != 0 && self["roomID"].current == 0
    }
}

impl Index<&str> for SNESState {
    type Output = MemoryWatcher;

    fn index(&self, var: &str) -> &Self::Output {
        self.vars.get(var).unwrap()
    }
}

impl MyApp {
    fn talk_to_snes(&mut self) -> std::result::Result<(), Box<dyn Error>> {
        let mut client = usb2snes::usb2snes::SyncClient::connect();
        client.set_name("annelid".to_owned())?;
        println!("Server version is {:?}", client.app_version());
        let mut devices = client.list_device()?;
        if devices.len() != 1 {
            if devices.len() < 1 {
                Err("No devices present")?;
            } else {
                Err(format!("You need to select a device: {:#?}", devices))?;
            }
        }
        let device = devices.pop().ok_or("Device list was empty")?;
        println!("Using device: {}", device);
        client.attach(&device)?;
        println!("Connected.");
        println!("{:#?}", client.info());
        {
            // controller
            let data = client.get_address(0xf5008b, 2)?;
            let raw_controller: u16 = (data[1] as u16) << 8 | data[0] as u16;
            let bbutton = (0x8000, "B");
            let ybutton = (0x4000, "Y");
            let selbutton = (0x2000, "Sl");
            let stbutton = (0x1000, "St");
            let ubutton = (0x800, "↑");
            let dbutton = (0x400, "↓");
            let leftbutton = (0x200, "←");
            let rightbutton = (0x100, "→");
            let abutton = (0x80, "A");
            let xbutton = (0x40, "X");
            let lbutton = (0x20, "L");
            let rbutton = (0x10, "R");
            let button_codes = vec![
                bbutton,
                ybutton,
                selbutton,
                stbutton,
                ubutton,
                dbutton,
                leftbutton,
                rightbutton,
                abutton,
                xbutton,
                lbutton,
                rbutton,
            ];
            let mut buttons = vec![];
            for code in button_codes.iter() {
                if raw_controller & code.0 > 0 {
                    buttons.push(code.1);
                }
            }
            print!("Controller 1: ");
            for b in buttons.iter() {
                print!("{} ", b);
            }
            println!("");
        }
        {
            // Room data
            let data = client.get_address(0xf5079b, 2)?;
            let room_ptr: u16 = (data[1] as u16) << 8 | data[0] as u16;
            println!("room pointer: 0x{:x}", room_ptr);
            let data = client.get_address(0xf5079d, 2)?;
            let room_index: u16 = (data[1] as u16) << 8 | data[0] as u16;
            println!("room index: 0x{:x}", room_index);
            let data = client.get_address(0xf5079f, 1)?;
            println!("area index: 0x{:x}", data[0]);
        }
        Ok(())
    }
    #[allow(non_snake_case)]
    fn split(&mut self) -> bool {
        // Ammo pickup section
        let firstMissile = self.settings.get("firstMissile")
            && self.snes["maxMissiles"].old == 0
            && self.snes["maxMissiles"].current == 5;
        let allMissiles = self.settings.get("allMissiles")
            && (self.snes["maxMissiles"].old + 5) == (self.snes["maxMissiles"].current);
        let oceanBottomMissiles = self.settings.get("oceanBottomMissiles")
            && self.snes["roomID"].current == roomIDEnum["westOcean"]
            && (self.snes["crateriaItems"].old + 2) == (self.snes["crateriaItems"].current);
        let oceanTopMissiles = self.settings.get("oceanTopMissiles")
            && self.snes["roomID"].current == roomIDEnum["westOcean"]
            && (self.snes["crateriaItems"].old + 4) == (self.snes["crateriaItems"].current);
        let oceanMiddleMissiles = self.settings.get("oceanMiddleMissiles")
            && self.snes["roomID"].current == roomIDEnum["westOcean"]
            && (self.snes["crateriaItems"].old + 8) == (self.snes["crateriaItems"].current);
        let moatMissiles = self.settings.get("moatMissiles")
            && self.snes["roomID"].current == roomIDEnum["crateriaMoat"]
            && (self.snes["crateriaItems"].old + 16) == (self.snes["crateriaItems"].current);
        let oldTourianMissiles = self.settings.get("oldTourianMissiles")
            && self.snes["roomID"].current == roomIDEnum["pitRoom"]
            && (self.snes["crateriaItems"].old + 64) == (self.snes["crateriaItems"].current);
        let gauntletRightMissiles = self.settings.get("gauntletRightMissiles")
            && self.snes["roomID"].current == roomIDEnum["greenPirateShaft"]
            && (self.snes["brinteriaItems"].old + 2) == (self.snes["brinteriaItems"].current);
        let gauntletLeftMissiles = self.settings.get("gauntletLeftMissiles")
            && self.snes["roomID"].current == roomIDEnum["greenPirateShaft"]
            && (self.snes["brinteriaItems"].old + 4) == (self.snes["brinteriaItems"].current);
        let dentalPlan = self.settings.get("dentalPlan")
            && self.snes["roomID"].current == roomIDEnum["theFinalMissile"]
            && (self.snes["brinteriaItems"].old + 16) == (self.snes["brinteriaItems"].current);
        let earlySuperBridgeMissiles = self.settings.get("earlySuperBridgeMissiles")
            && self.snes["roomID"].current == roomIDEnum["earlySupers"]
            && (self.snes["brinteriaItems"].old + 128) == (self.snes["brinteriaItems"].current);
        let greenBrinstarReserveMissiles = self.settings.get("greenBrinstarReserveMissiles")
            && self.snes["roomID"].current == roomIDEnum["brinstarReserveRoom"]
            && (self.snes["brinstarItems2"].old + 8) == (self.snes["brinstarItems2"].current);
        let greenBrinstarExtraReserveMissiles =
            self.settings.get("greenBrinstarExtraReserveMissiles")
                && self.snes["roomID"].current == roomIDEnum["brinstarReserveRoom"]
                && (self.snes["brinstarItems2"].old + 4) == (self.snes["brinstarItems2"].current);
        let bigPinkTopMissiles = self.settings.get("bigPinkTopMissiles")
            && self.snes["roomID"].current == roomIDEnum["bigPink"]
            && (self.snes["brinstarItems2"].old + 32) == (self.snes["brinstarItems2"].current);
        let chargeMissiles = self.settings.get("chargeMissiles")
            && self.snes["roomID"].current == roomIDEnum["bigPink"]
            && (self.snes["brinstarItems2"].old + 64) == (self.snes["brinstarItems2"].current);
        let greenHillsMissiles = self.settings.get("greenHillsMissiles")
            && self.snes["roomID"].current == roomIDEnum["greenHills"]
            && (self.snes["brinstarItems3"].old + 2) == (self.snes["brinstarItems3"].current);
        let blueBrinstarETankMissiles = self.settings.get("blueBrinstarETankMissiles")
            && self.snes["roomID"].current == roomIDEnum["blueBrinstarETankRoom"]
            && (self.snes["brinstarItems3"].old + 16) == (self.snes["brinstarItems3"].current);
        let alphaMissiles = self.settings.get("alphaMissiles")
            && self.snes["roomID"].current == roomIDEnum["alphaMissileRoom"]
            && (self.snes["brinstarItems4"].old + 4) == (self.snes["brinstarItems4"].current);
        let billyMaysMissiles = self.settings.get("billyMaysMissiles")
            && self.snes["roomID"].current == roomIDEnum["billyMays"]
            && (self.snes["brinstarItems4"].old + 16) == (self.snes["brinstarItems4"].current);
        let butWaitTheresMoreMissiles = self.settings.get("butWaitTheresMoreMissiles")
            && self.snes["roomID"].current == roomIDEnum["billyMays"]
            && (self.snes["brinstarItems4"].old + 32) == (self.snes["brinstarItems4"].current);
        let redBrinstarMissiles = self.settings.get("redBrinstarMissiles")
            && self.snes["roomID"].current == roomIDEnum["alphaPowerBombsRoom"]
            && (self.snes["brinstarItems5"].old + 2) == (self.snes["brinstarItems5"].current);
        let warehouseMissiles = self.settings.get("warehouseMissiles")
            && self.snes["roomID"].current == roomIDEnum["warehouseKiHunters"]
            && (self.snes["brinstarItems5"].old + 16) == (self.snes["brinstarItems5"].current);
        let cathedralMissiles = self.settings.get("cathedralMissiles")
            && self.snes["roomID"].current == roomIDEnum["cathedral"]
            && (self.snes["norfairItems1"].old + 2) == (self.snes["norfairItems1"].current);
        let crumbleShaftMissiles = self.settings.get("crumbleShaftMissiles")
            && self.snes["roomID"].current == roomIDEnum["crumbleShaft"]
            && (self.snes["norfairItems1"].old + 8) == (self.snes["norfairItems1"].current);
        let crocomireEscapeMissiles = self.settings.get("crocomireEscapeMissiles")
            && self.snes["roomID"].current == roomIDEnum["crocomireEscape"]
            && (self.snes["norfairItems1"].old + 64) == (self.snes["norfairItems1"].current);
        let hiJumpMissiles = self.settings.get("hiJumpMissiles")
            && self.snes["roomID"].current == roomIDEnum["hiJumpShaft"]
            && (self.snes["norfairItems1"].old + 128) == (self.snes["norfairItems1"].current);
        let postCrocomireMissiles = self.settings.get("postCrocomireMissiles")
            && self.snes["roomID"].current == roomIDEnum["cosineRoom"]
            && (self.snes["norfairItems2"].old + 4) == (self.snes["norfairItems2"].current);
        let grappleMissiles = self.settings.get("grappleMissiles")
            && self.snes["roomID"].current == roomIDEnum["preGrapple"]
            && (self.snes["norfairItems2"].old + 8) == (self.snes["norfairItems2"].current);
        let norfairReserveMissiles = self.settings.get("norfairReserveMissiles")
            && self.snes["roomID"].current == roomIDEnum["norfairReserveRoom"]
            && (self.snes["norfairItems2"].old + 64) == (self.snes["norfairItems2"].current);
        let greenBubblesMissiles = self.settings.get("greenBubblesMissiles")
            && self.snes["roomID"].current == roomIDEnum["greenBubblesRoom"]
            && (self.snes["norfairItems2"].old + 128) == (self.snes["norfairItems2"].current);
        let bubbleMountainMissiles = self.settings.get("bubbleMountainMissiles")
            && self.snes["roomID"].current == roomIDEnum["bubbleMountain"]
            && (self.snes["norfairItems3"].old + 1) == (self.snes["norfairItems3"].current);
        let speedBoostMissiles = self.settings.get("speedBoostMissiles")
            && self.snes["roomID"].current == roomIDEnum["speedBoostHall"]
            && (self.snes["norfairItems3"].old + 2) == (self.snes["norfairItems3"].current);
        let waveMissiles = self.settings.get("waveMissiles")
            && self.snes["roomID"].current == roomIDEnum["doubleChamber"]
            && (self.snes["norfairItems3"].old + 8) == (self.snes["norfairItems3"].current);
        let goldTorizoMissiles = self.settings.get("goldTorizoMissiles")
            && self.snes["roomID"].current == roomIDEnum["goldenTorizo"]
            && (self.snes["norfairItems3"].old + 64) == (self.snes["norfairItems3"].current);
        let mickeyMouseMissiles = self.settings.get("mickeyMouseMissiles")
            && self.snes["roomID"].current == roomIDEnum["mickeyMouse"]
            && (self.snes["norfairItems4"].old + 2) == (self.snes["norfairItems4"].current);
        let lowerNorfairSpringMazeMissiles = self.settings.get("lowerNorfairSpringMazeMissiles")
            && self.snes["roomID"].current == roomIDEnum["lowerNorfairSpringMaze"]
            && (self.snes["norfairItems4"].old + 4) == (self.snes["norfairItems4"].current);
        let threeMusketeersMissiles = self.settings.get("threeMusketeersMissiles")
            && self.snes["roomID"].current == roomIDEnum["threeMusketeers"]
            && (self.snes["norfairItems4"].old + 32) == (self.snes["norfairItems4"].current);
        let wreckedShipMainShaftMissiles = self.settings.get("wreckedShipMainShaftMissiles")
            && self.snes["roomID"].current == roomIDEnum["wreckedShipMainShaft"]
            && (self.snes["wreckedShipItems"].old + 1) == (self.snes["wreckedShipItems"].current);
        let bowlingMissiles = self.settings.get("bowlingMissiles")
            && self.snes["roomID"].current == roomIDEnum["bowling"]
            && (self.snes["wreckedShipItems"].old + 4) == (self.snes["wreckedShipItems"].current);
        let atticMissiles = self.settings.get("atticMissiles")
            && self.snes["roomID"].current == roomIDEnum["atticWorkerRobotRoom"]
            && (self.snes["wreckedShipItems"].old + 8) == (self.snes["wreckedShipItems"].current);
        let mainStreetMissiles = self.settings.get("mainStreetMissiles")
            && self.snes["roomID"].current == roomIDEnum["mainStreet"]
            && (self.snes["maridiaItems1"].old + 1) == (self.snes["maridiaItems1"].current);
        let mamaTurtleMissiles = self.settings.get("mamaTurtleMissiles")
            && self.snes["roomID"].current == roomIDEnum["mamaTurtle"]
            && (self.snes["maridiaItems1"].old + 8) == (self.snes["maridiaItems1"].current);
        let wateringHoleMissiles = self.settings.get("wateringHoleMissiles")
            && self.snes["roomID"].current == roomIDEnum["wateringHole"]
            && (self.snes["maridiaItems1"].old + 32) == (self.snes["maridiaItems1"].current);
        let beachMissiles = self.settings.get("beachMissiles")
            && self.snes["roomID"].current == roomIDEnum["beach"]
            && (self.snes["maridiaItems1"].old + 64) == (self.snes["maridiaItems1"].current);
        let leftSandPitMissiles = self.settings.get("leftSandPitMissiles")
            && self.snes["roomID"].current == roomIDEnum["leftSandPit"]
            && (self.snes["maridiaItems2"].old + 1) == (self.snes["maridiaItems2"].current);
        let rightSandPitMissiles = self.settings.get("rightSandPitMissiles")
            && self.snes["roomID"].current == roomIDEnum["rightSandPit"]
            && (self.snes["maridiaItems2"].old + 4) == (self.snes["maridiaItems2"].current);
        let aqueductMissiles = self.settings.get("aqueductMissiles")
            && self.snes["roomID"].current == roomIDEnum["aqueduct"]
            && (self.snes["maridiaItems2"].old + 16) == (self.snes["maridiaItems2"].current);
        let preDraygonMissiles = self.settings.get("preDraygonMissiles")
            && self.snes["roomID"].current == roomIDEnum["precious"]
            && (self.snes["maridiaItems2"].old + 128) == (self.snes["maridiaItems2"].current);
        let firstSuper = self.settings.get("firstSuper")
            && self.snes["maxSupers"].old == 0
            && self.snes["maxSupers"].current == 5;
        let allSupers = self.settings.get("allSupers")
            && (self.snes["maxSupers"].old + 5) == (self.snes["maxSupers"].current);
        let climbSupers = self.settings.get("climbSupers")
            && self.snes["roomID"].current == roomIDEnum["crateriaSupersRoom"]
            && (self.snes["brinteriaItems"].old + 8) == (self.snes["brinteriaItems"].current);
        let sporeSpawnSupers = self.settings.get("sporeSpawnSupers")
            && self.snes["roomID"].current == roomIDEnum["sporeSpawnSuper"]
            && (self.snes["brinteriaItems"].old + 64) == (self.snes["brinteriaItems"].current);
        let earlySupers = self.settings.get("earlySupers")
            && self.snes["roomID"].current == roomIDEnum["earlySupers"]
            && (self.snes["brinstarItems2"].old + 1) == (self.snes["brinstarItems2"].current);
        let etacoonSupers = self.settings.get("etacoonSupers")
            && self.snes["roomID"].current == roomIDEnum["etacoonSuperRoom"]
            && (self.snes["brinstarItems3"].old + 128) == (self.snes["brinstarItems3"].current);
        let goldTorizoSupers = self.settings.get("goldTorizoSupers")
            && self.snes["roomID"].current == roomIDEnum["goldenTorizo"]
            && (self.snes["norfairItems3"].old + 128) == (self.snes["norfairItems3"].current);
        let wreckedShipLeftSupers = self.settings.get("wreckedShipLeftSupers")
            && self.snes["roomID"].current == roomIDEnum["wreckedShipLeftSuperRoom"]
            && (self.snes["wreckedShipItems"].old + 32) == (self.snes["wreckedShipItems"].current);
        let wreckedShipRightSupers = self.settings.get("wreckedShipRightSupers")
            && self.snes["roomID"].current == roomIDEnum["wreckedShipRightSuperRoom"]
            && (self.snes["wreckedShipItems"].old + 64) == (self.snes["wreckedShipItems"].current);
        let crabSupers = self.settings.get("crabSupers")
            && self.snes["roomID"].current == roomIDEnum["mainStreet"]
            && (self.snes["maridiaItems1"].old + 2) == (self.snes["maridiaItems1"].current);
        let wateringHoleSupers = self.settings.get("wateringHoleSupers")
            && self.snes["roomID"].current == roomIDEnum["wateringHole"]
            && (self.snes["maridiaItems1"].old + 16) == (self.snes["maridiaItems1"].current);
        let aqueductSupers = self.settings.get("aqueductSupers")
            && self.snes["roomID"].current == roomIDEnum["aqueduct"]
            && (self.snes["maridiaItems2"].old + 32) == (self.snes["maridiaItems2"].current);
        let firstPowerBomb = self.settings.get("firstPowerBomb")
            && self.snes["maxPowerBombs"].old == 0
            && self.snes["maxPowerBombs"].current == 5;
        let allPowerBombs = self.settings.get("allPowerBombs")
            && (self.snes["maxPowerBombs"].old + 5) == (self.snes["maxPowerBombs"].current);
        let landingSiteBombs = self.settings.get("landingSiteBombs")
            && self.snes["roomID"].current == roomIDEnum["crateriaPowerBombRoom"]
            && (self.snes["crateriaItems"].old + 1) == (self.snes["crateriaItems"].current);
        let etacoonBombs = self.settings.get("etacoonBombs")
            && self.snes["roomID"].current == roomIDEnum["greenBrinstarMainShaft"]
            && (self.snes["brinteriaItems"].old + 32) == (self.snes["brinteriaItems"].current);
        let pinkBrinstarBombs = self.settings.get("pinkBrinstarBombs")
            && self.snes["roomID"].current == roomIDEnum["pinkBrinstarPowerBombRoom"]
            && (self.snes["brinstarItems3"].old + 1) == (self.snes["brinstarItems3"].current);
        let blueBrinstarBombs = self.settings.get("blueBrinstarBombs")
            && self.snes["roomID"].current == roomIDEnum["morphBall"]
            && (self.snes["brinstarItems3"].old + 8) == (self.snes["brinstarItems3"].current);
        let alphaBombs = self.settings.get("alphaBombs")
            && self.snes["roomID"].current == roomIDEnum["alphaPowerBombsRoom"]
            && (self.snes["brinstarItems5"].old + 1) == (self.snes["brinstarItems5"].current);
        let betaBombs = self.settings.get("betaBombs")
            && self.snes["roomID"].current == roomIDEnum["betaPowerBombRoom"]
            && (self.snes["brinstarItems4"].old + 128) == (self.snes["brinstarItems4"].current);
        let crocomireBombs = self.settings.get("crocomireBombs")
            && self.snes["roomID"].current == roomIDEnum["postCrocomirePowerBombRoom"]
            && (self.snes["norfairItems2"].old + 2) == (self.snes["norfairItems2"].current);
        let lowerNorfairEscapeBombs = self.settings.get("lowerNorfairEscapeBombs")
            && self.snes["roomID"].current == roomIDEnum["lowerNorfairEscapePowerBombRoom"]
            && (self.snes["norfairItems4"].old + 8) == (self.snes["norfairItems4"].current);
        let shameBombs = self.settings.get("shameBombs")
            && self.snes["roomID"].current == roomIDEnum["wasteland"]
            && (self.snes["norfairItems4"].old + 16) == (self.snes["norfairItems4"].current);
        let rightSandPitBombs = self.settings.get("rightSandPitBombs")
            && self.snes["roomID"].current == roomIDEnum["rightSandPit"]
            && (self.snes["maridiaItems2"].old + 8) == (self.snes["maridiaItems2"].current);
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
            || etacoonSupers
            || goldTorizoSupers
            || wreckedShipLeftSupers
            || wreckedShipRightSupers
            || crabSupers
            || wateringHoleSupers
            || aqueductSupers
            || firstPowerBomb
            || allPowerBombs
            || landingSiteBombs
            || etacoonBombs
            || pinkBrinstarBombs
            || blueBrinstarBombs
            || alphaBombs
            || betaBombs
            || crocomireBombs
            || lowerNorfairEscapeBombs
            || shameBombs
            || rightSandPitBombs;

        // Item unlock section
        let varia = self.settings.get("variaSuit")
            && self.snes["roomID"].current == roomIDEnum["varia"]
            && (self.snes["unlockedEquips2"].old & unlockFlagEnum["variaSuit"]) == 0
            && (self.snes["unlockedEquips2"].current & unlockFlagEnum["variaSuit"]) > 0;
        let springBall = self.settings.get("springBall")
            && self.snes["roomID"].current == roomIDEnum["springBall"]
            && (self.snes["unlockedEquips2"].old & unlockFlagEnum["springBall"]) == 0
            && (self.snes["unlockedEquips2"].current & unlockFlagEnum["springBall"]) > 0;
        let morphBall = self.settings.get("morphBall")
            && self.snes["roomID"].current == roomIDEnum["morphBall"]
            && (self.snes["unlockedEquips2"].old & unlockFlagEnum["morphBall"]) == 0
            && (self.snes["unlockedEquips2"].current & unlockFlagEnum["morphBall"]) > 0;
        let screwAttack = self.settings.get("screwAttack")
            && self.snes["roomID"].current == roomIDEnum["screwAttack"]
            && (self.snes["unlockedEquips2"].old & unlockFlagEnum["screwAttack"]) == 0
            && (self.snes["unlockedEquips2"].current & unlockFlagEnum["screwAttack"]) > 0;
        let gravSuit = self.settings.get("gravSuit")
            && self.snes["roomID"].current == roomIDEnum["gravity"]
            && (self.snes["unlockedEquips2"].old & unlockFlagEnum["gravSuit"]) == 0
            && (self.snes["unlockedEquips2"].current & unlockFlagEnum["gravSuit"]) > 0;
        let hiJump = self.settings.get("hiJump")
            && self.snes["roomID"].current == roomIDEnum["hiJump"]
            && (self.snes["unlockedEquips"].old & unlockFlagEnum["hiJump"]) == 0
            && (self.snes["unlockedEquips"].current & unlockFlagEnum["hiJump"]) > 0;
        let spaceJump = self.settings.get("spaceJump")
            && self.snes["roomID"].current == roomIDEnum["spaceJump"]
            && (self.snes["unlockedEquips"].old & unlockFlagEnum["spaceJump"]) == 0
            && (self.snes["unlockedEquips"].current & unlockFlagEnum["spaceJump"]) > 0;
        let bomb = self.settings.get("bomb")
            && self.snes["roomID"].current == roomIDEnum["bombTorizo"]
            && (self.snes["unlockedEquips"].old & unlockFlagEnum["bomb"]) == 0
            && (self.snes["unlockedEquips"].current & unlockFlagEnum["bomb"]) > 0;
        let speedBooster = self.settings.get("speedBooster")
            && self.snes["roomID"].current == roomIDEnum["speedBooster"]
            && (self.snes["unlockedEquips"].old & unlockFlagEnum["speedBooster"]) == 0
            && (self.snes["unlockedEquips"].current & unlockFlagEnum["speedBooster"]) > 0;
        let grapple = self.settings.get("grapple")
            && self.snes["roomID"].current == roomIDEnum["grapple"]
            && (self.snes["unlockedEquips"].old & unlockFlagEnum["grapple"]) == 0
            && (self.snes["unlockedEquips"].current & unlockFlagEnum["grapple"]) > 0;
        let xray = self.settings.get("xray")
            && self.snes["roomID"].current == roomIDEnum["xRay"]
            && (self.snes["unlockedEquips"].old & unlockFlagEnum["xray"]) == 0
            && (self.snes["unlockedEquips"].current & unlockFlagEnum["xray"]) > 0;
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
        let wave = self.settings.get("wave")
            && self.snes["roomID"].current == roomIDEnum["waveBeam"]
            && (self.snes["unlockedBeams"].old & unlockFlagEnum["wave"]) == 0
            && (self.snes["unlockedBeams"].current & unlockFlagEnum["wave"]) > 0;
        let ice = self.settings.get("ice")
            && self.snes["roomID"].current == roomIDEnum["iceBeam"]
            && (self.snes["unlockedBeams"].old & unlockFlagEnum["ice"]) == 0
            && (self.snes["unlockedBeams"].current & unlockFlagEnum["ice"]) > 0;
        let spazer = self.settings.get("spazer")
            && self.snes["roomID"].current == roomIDEnum["spazer"]
            && (self.snes["unlockedBeams"].old & unlockFlagEnum["spazer"]) == 0
            && (self.snes["unlockedBeams"].current & unlockFlagEnum["spazer"]) > 0;
        let plasma = self.settings.get("plasma")
            && self.snes["roomID"].current == roomIDEnum["plasmaBeam"]
            && (self.snes["unlockedBeams"].old & unlockFlagEnum["plasma"]) == 0
            && (self.snes["unlockedBeams"].current & unlockFlagEnum["plasma"]) > 0;
        let chargeBeam = self.settings.get("chargeBeam")
            && self.snes["roomID"].current == roomIDEnum["bigPink"]
            && (self.snes["unlockedCharge"].old & unlockFlagEnum["chargeBeam"]) == 0
            && (self.snes["unlockedCharge"].current & unlockFlagEnum["chargeBeam"]) > 0;
        let beam = wave || ice || spazer || plasma || chargeBeam;

        // E-tanks and reserve tanks
        let firstETank = self.settings.get("firstETank")
            && self.snes["maxEnergy"].old == 99
            && self.snes["maxEnergy"].current == 199;
        let allETanks = self.settings.get("allETanks")
            && (self.snes["maxEnergy"].old + 100) == (self.snes["maxEnergy"].current);
        let gauntletETank = self.settings.get("gauntletETank")
            && self.snes["roomID"].current == roomIDEnum["gauntletETankRoom"]
            && (self.snes["crateriaItems"].old + 32) == (self.snes["crateriaItems"].current);
        let terminatorETank = self.settings.get("terminatorETank")
            && self.snes["roomID"].current == roomIDEnum["terminator"]
            && (self.snes["brinteriaItems"].old + 1) == (self.snes["brinteriaItems"].current);
        let ceilingETank = self.settings.get("ceilingETank")
            && self.snes["roomID"].current == roomIDEnum["blueBrinstarETankRoom"]
            && (self.snes["brinstarItems3"].old + 32) == (self.snes["brinstarItems3"].current);
        let etecoonsETank = self.settings.get("etecoonsETank")
            && self.snes["roomID"].current == roomIDEnum["etacoonETankRoom"]
            && (self.snes["brinstarItems3"].old + 64) == (self.snes["brinstarItems3"].current);
        let waterwayETank = self.settings.get("waterwayETank")
            && self.snes["roomID"].current == roomIDEnum["waterway"]
            && (self.snes["brinstarItems4"].old + 2) == (self.snes["brinstarItems4"].current);
        let waveGateETank = self.settings.get("waveGateETank")
            && self.snes["roomID"].current == roomIDEnum["hopperETankRoom"]
            && (self.snes["brinstarItems4"].old + 8) == (self.snes["brinstarItems4"].current);
        let kraidETank = self.settings.get("kraidETank")
            && self.snes["roomID"].current == roomIDEnum["warehouseETankRoom"]
            && (self.snes["brinstarItems5"].old + 8) == (self.snes["brinstarItems5"].current);
        let crocomireETank = self.settings.get("crocomireETank")
            && self.snes["roomID"].current == roomIDEnum["crocomire"]
            && (self.snes["norfairItems1"].old + 16) == (self.snes["norfairItems1"].current);
        let hiJumpETank = self.settings.get("hiJumpETank")
            && self.snes["roomID"].current == roomIDEnum["hiJumpShaft"]
            && (self.snes["norfairItems2"].old + 1) == (self.snes["norfairItems2"].current);
        let ridleyETank = self.settings.get("ridleyETank")
            && self.snes["roomID"].current == roomIDEnum["ridleyETankRoom"]
            && (self.snes["norfairItems4"].old + 64) == (self.snes["norfairItems4"].current);
        let firefleaETank = self.settings.get("firefleaETank")
            && self.snes["roomID"].current == roomIDEnum["lowerNorfairFireflea"]
            && (self.snes["norfairItems5"].old + 1) == (self.snes["norfairItems5"].current);
        let wreckedShipETank = self.settings.get("wreckedShipETank")
            && self.snes["roomID"].current == roomIDEnum["wreckedShipETankRoom"]
            && (self.snes["wreckedShipItems"].old + 16) == (self.snes["wreckedShipItems"].current);
        let tatoriETank = self.settings.get("tatoriETank")
            && self.snes["roomID"].current == roomIDEnum["mamaTurtle"]
            && (self.snes["maridiaItems1"].old + 4) == (self.snes["maridiaItems1"].current);
        let botwoonETank = self.settings.get("botwoonETank")
            && self.snes["roomID"].current == roomIDEnum["botwoonETankRoom"]
            && (self.snes["maridiaItems3"].old + 1) == (self.snes["maridiaItems3"].current);
        let reserveTanks = self.settings.get("reserveTanks")
            && (self.snes["maxReserve"].old + 100) == (self.snes["maxReserve"].current);
        let brinstarReserve = self.settings.get("brinstarReserve")
            && self.snes["roomID"].current == roomIDEnum["brinstarReserveRoom"]
            && (self.snes["brinstarItems2"].old + 2) == (self.snes["brinstarItems2"].current);
        let norfairReserve = self.settings.get("norfairReserve")
            && self.snes["roomID"].current == roomIDEnum["norfairReserveRoom"]
            && (self.snes["norfairItems2"].old + 32) == (self.snes["norfairItems2"].current);
        let wreckedShipReserve = self.settings.get("wreckedShipReserve")
            && self.snes["roomID"].current == roomIDEnum["bowling"]
            && (self.snes["wreckedShipItems"].old + 2) == (self.snes["wreckedShipItems"].current);
        let maridiaReserve = self.settings.get("maridiaReserve")
            && self.snes["roomID"].current == roomIDEnum["leftSandPit"]
            && (self.snes["maridiaItems2"].old + 2) == (self.snes["maridiaItems2"].current);
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
        if self.settings.get("miniBossRooms") {
            let ceresRidleyRoom = self.snes["roomID"].old == roomIDEnum["flatRoom"]
                && self.snes["roomID"].current == roomIDEnum["ceresRidley"];
            let sporeSpawnRoom = self.snes["roomID"].old == roomIDEnum["sporeSpawnKeyhunter"]
                && self.snes["roomID"].current == roomIDEnum["sporeSpawn"];
            let crocomireRoom = self.snes["roomID"].old == roomIDEnum["crocomireSpeedway"]
                && self.snes["roomID"].current == roomIDEnum["crocomire"];
            let botwoonRoom = self.snes["roomID"].old == roomIDEnum["botwoonHallway"]
                && self.snes["roomID"].current == roomIDEnum["botwoon"];
            // Allow either vanilla or GGG entry
            let goldenTorizoRoom = (self.snes["roomID"].old == roomIDEnum["acidStatue"]
                || self.snes["roomID"].old == roomIDEnum["screwAttack"])
                && self.snes["roomID"].current == roomIDEnum["goldenTorizo"];
            miniBossRooms = ceresRidleyRoom
                || sporeSpawnRoom
                || crocomireRoom
                || botwoonRoom
                || goldenTorizoRoom;
        }

        // Boss room transitions
        let mut bossRooms = false;
        if self.settings.get("bossRooms") {
            let kraidRoom = self.snes["roomID"].old == roomIDEnum["kraidEyeDoor"]
                && self.snes["roomID"].current == roomIDEnum["kraid"];
            let phantoonRoom = self.snes["roomID"].old == roomIDEnum["basement"]
                && self.snes["roomID"].current == roomIDEnum["phantoon"];
            let draygonRoom = self.snes["roomID"].old == roomIDEnum["precious"]
                && self.snes["roomID"].current == roomIDEnum["draygon"];
            let ridleyRoom = self.snes["roomID"].old == roomIDEnum["lowerNorfairFarming"]
                && self.snes["roomID"].current == roomIDEnum["ridley"];
            let motherBrainRoom = self.snes["roomID"].old == roomIDEnum["rinkaShaft"]
                && self.snes["roomID"].current == roomIDEnum["motherBrain"];
            bossRooms = kraidRoom || phantoonRoom || draygonRoom || ridleyRoom || motherBrainRoom;
        }

        // Elevator transitions between areas
        let mut elevatorTransitions = false;
        if self.settings.get("elevatorTransitions") {
            let blueBrinstar = (self.snes["roomID"].old == roomIDEnum["elevatorToMorphBall"]
                && self.snes["roomID"].current == roomIDEnum["morphBall"])
                || (self.snes["roomID"].old == roomIDEnum["morphBall"]
                    && self.snes["roomID"].current == roomIDEnum["elevatorToMorphBall"]);
            let greenBrinstar = (self.snes["roomID"].old == roomIDEnum["elevatorToGreenBrinstar"]
                && self.snes["roomID"].current == roomIDEnum["greenBrinstarMainShaft"])
                || (self.snes["roomID"].old == roomIDEnum["greenBrinstarMainShaft"]
                    && self.snes["roomID"].current == roomIDEnum["elevatorToGreenBrinstar"]);
            let businessCenter = (self.snes["roomID"].old == roomIDEnum["warehouseEntrance"]
                && self.snes["roomID"].current == roomIDEnum["businessCenter"])
                || (self.snes["roomID"].old == roomIDEnum["businessCenter"]
                    && self.snes["roomID"].current == roomIDEnum["warehouseEntrance"]);
            let caterpillar = (self.snes["roomID"].old == roomIDEnum["elevatorToCaterpillar"]
                && self.snes["roomID"].current == roomIDEnum["caterpillar"])
                || (self.snes["roomID"].old == roomIDEnum["caterpillar"]
                    && self.snes["roomID"].current == roomIDEnum["elevatorToCaterpillar"]);
            let maridiaElevator = (self.snes["roomID"].old == roomIDEnum["elevatorToMaridia"]
                && self.snes["roomID"].current == roomIDEnum["maridiaElevator"])
                || (self.snes["roomID"].old == roomIDEnum["maridiaElevator"]
                    && self.snes["roomID"].current == roomIDEnum["elevatorToMaridia"]);
            elevatorTransitions =
                blueBrinstar || greenBrinstar || businessCenter || caterpillar || maridiaElevator;
        }

        // Room transitions
        let ceresEscape = self.settings.get("ceresEscape")
            && self.snes["roomID"].current == roomIDEnum["ceresElevator"]
            && self.snes["gameState"].old == gameStateEnum["normalGameplay"]
            && self.snes["gameState"].current == gameStateEnum["startOfCeresCutscene"];
        let wreckedShipEntrance = self.settings.get("wreckedShipEntrance")
            && self.snes["roomID"].old == roomIDEnum["westOcean"]
            && self.snes["roomID"].current == roomIDEnum["wreckedShipEntrance"];
        let redTowerMiddleEntrance = self.settings.get("redTowerMiddleEntrance")
            && self.snes["roomID"].old == roomIDEnum["noobBridge"]
            && self.snes["roomID"].current == roomIDEnum["redTower"];
        let redTowerBottomEntrance = self.settings.get("redTowerBottomEntrance")
            && self.snes["roomID"].old == roomIDEnum["bat"]
            && self.snes["roomID"].current == roomIDEnum["redTower"];
        let kraidsLair = self.settings.get("kraidsLair")
            && self.snes["roomID"].old == roomIDEnum["warehouseEntrance"]
            && self.snes["roomID"].current == roomIDEnum["warehouseZeela"];
        let risingTideEntrance = self.settings.get("risingTideEntrance")
            && self.snes["roomID"].old == roomIDEnum["cathedral"]
            && self.snes["roomID"].current == roomIDEnum["risingTide"];
        let atticExit = self.settings.get("atticExit")
            && self.snes["roomID"].old == roomIDEnum["attic"]
            && self.snes["roomID"].current == roomIDEnum["westOcean"];
        let tubeBroken = self.settings.get("tubeBroken")
            && self.snes["roomID"].current == roomIDEnum["glassTunnel"]
            && (self.snes["eventFlags"].old & eventFlagEnum["tubeBroken"]) == 0
            && (self.snes["eventFlags"].current & eventFlagEnum["tubeBroken"]) > 0;
        let cacExit = self.settings.get("cacExit")
            && self.snes["roomID"].old == roomIDEnum["westCactusAlley"]
            && self.snes["roomID"].current == roomIDEnum["butterflyRoom"];
        let toilet = self.settings.get("toilet")
            && (self.snes["roomID"].old == roomIDEnum["plasmaSpark"]
                && self.snes["roomID"].current == roomIDEnum["toiletBowl"]
                || self.snes["roomID"].old == roomIDEnum["oasis"]
                    && self.snes["roomID"].current == roomIDEnum["toiletBowl"]);
        let kronicBoost = self.settings.get("kronicBoost")
            && (self.snes["roomID"].old == roomIDEnum["magdolliteTunnel"]
                && self.snes["roomID"].current == roomIDEnum["kronicBoost"]
                || self.snes["roomID"].old == roomIDEnum["spikyAcidSnakes"]
                    && self.snes["roomID"].current == roomIDEnum["kronicBoost"]
                || self.snes["roomID"].old == roomIDEnum["volcano"]
                    && self.snes["roomID"].current == roomIDEnum["kronicBoost"]);
        let lowerNorfairEntrance = self.settings.get("lowerNorfairEntrance")
            && self.snes["roomID"].old == roomIDEnum["lowerNorfairElevator"]
            && self.snes["roomID"].current == roomIDEnum["mainHall"];
        let writg = self.settings.get("writg")
            && self.snes["roomID"].old == roomIDEnum["pillars"]
            && self.snes["roomID"].current == roomIDEnum["writg"];
        let redKiShaft = self.settings.get("redKiShaft")
            && (self.snes["roomID"].old == roomIDEnum["amphitheatre"]
                && self.snes["roomID"].current == roomIDEnum["redKiShaft"]
                || self.snes["roomID"].old == roomIDEnum["wasteland"]
                    && self.snes["roomID"].current == roomIDEnum["redKiShaft"]);
        let metalPirates = self.settings.get("metalPirates")
            && self.snes["roomID"].old == roomIDEnum["wasteland"]
            && self.snes["roomID"].current == roomIDEnum["metalPirates"];
        let lowerNorfairSpringMaze = self.settings.get("lowerNorfairSpringMaze")
            && self.snes["roomID"].old == roomIDEnum["lowerNorfairFireflea"]
            && self.snes["roomID"].current == roomIDEnum["lowerNorfairSpringMaze"];
        let lowerNorfairExit = self.settings.get("lowerNorfairExit")
            && self.snes["roomID"].old == roomIDEnum["threeMusketeers"]
            && self.snes["roomID"].current == roomIDEnum["singleChamber"];
        let allBossesFinished = (self.snes["brinstarBosses"].current & bossFlagEnum["kraid"]) > 0
            && (self.snes["wreckedShipBosses"].current & bossFlagEnum["phantoon"]) > 0
            && (self.snes["maridiaBosses"].current & bossFlagEnum["draygon"]) > 0
            && (self.snes["norfairBosses"].current & bossFlagEnum["ridley"]) > 0;
        let goldenFour = self.settings.get("goldenFour")
            && self.snes["roomID"].old == roomIDEnum["statuesHallway"]
            && self.snes["roomID"].current == roomIDEnum["statues"]
            && allBossesFinished;
        let tourianEntrance = self.settings.get("tourianEntrance")
            && self.snes["roomID"].old == roomIDEnum["statues"]
            && self.snes["roomID"].current == roomIDEnum["tourianElevator"];
        let metroids = self.settings.get("metroids")
            && (self.snes["roomID"].old == roomIDEnum["metroidOne"]
                && self.snes["roomID"].current == roomIDEnum["metroidTwo"]
                || self.snes["roomID"].old == roomIDEnum["metroidTwo"]
                    && self.snes["roomID"].current == roomIDEnum["metroidThree"]
                || self.snes["roomID"].old == roomIDEnum["metroidThree"]
                    && self.snes["roomID"].current == roomIDEnum["metroidFour"]
                || self.snes["roomID"].old == roomIDEnum["metroidFour"]
                    && self.snes["roomID"].current == roomIDEnum["tourianHopper"]);
        let babyMetroidRoom = self.settings.get("babyMetroidRoom")
            && self.snes["roomID"].old == roomIDEnum["dustTorizo"]
            && self.snes["roomID"].current == roomIDEnum["bigBoy"];
        let escapeClimb = self.settings.get("escapeClimb")
            && self.snes["roomID"].old == roomIDEnum["tourianEscape4"]
            && self.snes["roomID"].current == roomIDEnum["climb"];
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
        let ceresRidley = self.settings.get("ceresRidley")
            && (self.snes["ceresBosses"].old & bossFlagEnum["ceresRidley"]) == 0
            && (self.snes["ceresBosses"].current & bossFlagEnum["ceresRidley"]) > 0
            && self.snes["roomID"].current == roomIDEnum["ceresRidley"];
        let bombTorizo = self.settings.get("bombTorizo")
            && (self.snes["crateriaBosses"].old & bossFlagEnum["bombTorizo"]) == 0
            && (self.snes["crateriaBosses"].current & bossFlagEnum["bombTorizo"]) > 0
            && self.snes["roomID"].current == roomIDEnum["bombTorizo"];
        let sporeSpawn = self.settings.get("sporeSpawn")
            && (self.snes["brinstarBosses"].old & bossFlagEnum["sporeSpawn"]) == 0
            && (self.snes["brinstarBosses"].current & bossFlagEnum["sporeSpawn"]) > 0
            && self.snes["roomID"].current == roomIDEnum["sporeSpawn"];
        let crocomire = self.settings.get("crocomire")
            && (self.snes["norfairBosses"].old & bossFlagEnum["crocomire"]) == 0
            && (self.snes["norfairBosses"].current & bossFlagEnum["crocomire"]) > 0
            && self.snes["roomID"].current == roomIDEnum["crocomire"];
        let botwoon = self.settings.get("botwoon")
            && (self.snes["maridiaBosses"].old & bossFlagEnum["botwoon"]) == 0
            && (self.snes["maridiaBosses"].current & bossFlagEnum["botwoon"]) > 0
            && self.snes["roomID"].current == roomIDEnum["botwoon"];
        let goldenTorizo = self.settings.get("goldenTorizo")
            && (self.snes["norfairBosses"].old & bossFlagEnum["goldenTorizo"]) == 0
            && (self.snes["norfairBosses"].current & bossFlagEnum["goldenTorizo"]) > 0
            && self.snes["roomID"].current == roomIDEnum["goldenTorizo"];
        let minibossDefeat =
            ceresRidley || bombTorizo || sporeSpawn || crocomire || botwoon || goldenTorizo;

        // Bosses
        let kraid = self.settings.get("kraid")
            && (self.snes["brinstarBosses"].old & bossFlagEnum["kraid"]) == 0
            && (self.snes["brinstarBosses"].current & bossFlagEnum["kraid"]) > 0
            && self.snes["roomID"].current == roomIDEnum["kraid"];
        if kraid {
            println!("Split due to kraid defeat");
        }
        let phantoon = self.settings.get("phantoon")
            && (self.snes["wreckedShipBosses"].old & bossFlagEnum["phantoon"]) == 0
            && (self.snes["wreckedShipBosses"].current & bossFlagEnum["phantoon"]) > 0
            && self.snes["roomID"].current == roomIDEnum["phantoon"];
        if phantoon {
            println!("Split due to phantoon defeat");
        }
        let draygon = self.settings.get("draygon")
            && (self.snes["maridiaBosses"].old & bossFlagEnum["draygon"]) == 0
            && (self.snes["maridiaBosses"].current & bossFlagEnum["draygon"]) > 0
            && self.snes["roomID"].current == roomIDEnum["draygon"];
        if draygon {
            println!("Split due to draygon defeat");
        }
        let ridley = self.settings.get("ridley")
            && (self.snes["norfairBosses"].old & bossFlagEnum["ridley"]) == 0
            && (self.snes["norfairBosses"].current & bossFlagEnum["ridley"]) > 0
            && self.snes["roomID"].current == roomIDEnum["ridley"];
        if ridley {
            println!("Split due to ridley defeat");
        }
        // Mother Brain phases
        let inMotherBrainRoom = self.snes["roomID"].current == roomIDEnum["motherBrain"];
        let mb1 = self.settings.get("mb1")
            && inMotherBrainRoom
            && self.snes["gameState"].current == gameStateEnum["normalGameplay"]
            && self.snes["motherBrainHP"].old == 0
            && self.snes["motherBrainHP"].current == (motherBrainMaxHPEnum["phase2"]);
        if mb1 {
            println!("Split due to mb1 defeat");
        }
        let mb2 = self.settings.get("mb2")
            && inMotherBrainRoom
            && self.snes["gameState"].current == gameStateEnum["normalGameplay"]
            && self.snes["motherBrainHP"].old == 0
            && self.snes["motherBrainHP"].current == (motherBrainMaxHPEnum["phase3"]);
        if mb2 {
            println!("Split due to mb2 defeat");
        }
        let mb3 = self.settings.get("mb3")
            && inMotherBrainRoom
            && (self.snes["tourianBosses"].old & bossFlagEnum["motherBrain"]) == 0
            && (self.snes["tourianBosses"].current & bossFlagEnum["motherBrain"]) > 0;
        if mb3 {
            println!("Split due to mb3 defeat");
        }
        let bossDefeat = kraid || phantoon || draygon || ridley || mb1 || mb2 || mb3;

        // Run-ending splits
        let escape = self.settings.get("rtaFinish")
            && (self.snes["eventFlags"].current & eventFlagEnum["zebesAblaze"]) > 0
            && self.snes["shipAI"].old != 0xaa4f
            && self.snes["shipAI"].current == 0xaa4f;

        let takeoff = self.settings.get("igtFinish")
            && self.snes["roomID"].current == roomIDEnum["landingSite"]
            && self.snes["gameState"].old == gameStateEnum["preEndCutscene"]
            && self.snes["gameState"].current == gameStateEnum["endCutscene"];

        let mut sporeSpawnRTAFinish = false;
        if self.settings.get("sporeSpawnRTAFinish") {
            if self.pickedUpSporeSpawnSuper {
                if self.snes["igtFrames"].old != self.snes["igtFrames"].current {
                    sporeSpawnRTAFinish = true;
                    self.pickedUpSporeSpawnSuper = false;
                }
            } else {
                self.pickedUpSporeSpawnSuper = self.snes["roomID"].current
                    == roomIDEnum["sporeSpawnSuper"]
                    && (self.snes["maxSupers"].old + 5) == (self.snes["maxSupers"].current)
                    && (self.snes["brinstarBosses"].current & bossFlagEnum["sporeSpawn"]) > 0;
            }
        }

        let mut hundredMissileRTAFinish = false;
        if self.settings.get("hundredMissileRTAFinish") {
            if self.pickedUpHundredthMissile {
                if self.snes["igtFrames"].old != self.snes["igtFrames"].current {
                    hundredMissileRTAFinish = true;
                    self.pickedUpHundredthMissile = false;
                }
            } else {
                self.pickedUpHundredthMissile =
                    self.snes["maxMissiles"].old == 95 && self.snes["maxMissiles"].current == 100;
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

        return pickup
            || unlock
            || beam
            || energyUpgrade
            || roomTransitions
            || minibossDefeat
            || bossDefeat
            || escape
            || takeoff
            || nonStandardCategoryFinish;
    }
}

fn mk_text(text: &str, size: f32, color: egui::Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.halign = egui::Align::LEFT;
    job.justify = false;
    job.wrap.max_width = f32::INFINITY;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.append(
        text,
        0.0,
        epaint::text::TextFormat {
            font_id: epaint::text::FontId {
                size: size,
                family: epaint::text::FontFamily::Proportional,
            },
            color: color,
            ..epaint::text::TextFormat::default()
        },
    );
    job
}

#[inline]
pub fn columns<R>(
    ui: &mut egui::Ui,
    cols: &[(f32, egui::Layout)],
    add_contents: impl FnOnce(&mut [egui::Ui]) -> R,
) -> R {
    columns_dyn(ui, cols, Box::new(add_contents))
}

fn columns_dyn<'c, R>(
    ui: &mut egui::Ui,
    cols: &[(f32, egui::Layout)],
    add_contents: Box<dyn FnOnce(&mut [egui::Ui]) -> R + 'c>,
) -> R {
    // TODO: ensure there is space
    let spacing = ui.spacing().item_spacing.x;
    let top_left = ui.cursor().min;

    let mut total_width = 0.0;
    let mut columns: Vec<egui::Ui> = vec![];
    for (column_width, col_layout) in cols.iter() {
        let pos = top_left + egui::vec2(total_width, 0.0);
        let child_rect = egui::Rect::from_min_max(
            pos,
            egui::pos2(pos.x + column_width, ui.max_rect().right_bottom().y),
        );
        let mut column_ui = ui.child_ui(child_rect, *col_layout);
        column_ui.set_width(*column_width);
        //total_width += column_ui.min_rect().width();
        total_width += column_width + spacing;
        columns.push(column_ui);
    }

    let result = add_contents(&mut columns[..]);

    let mut max_column_width = cols[0].0;
    let mut max_height = 0.0;
    for column in &columns {
        max_column_width = max_column_width.max(column.min_rect().width());
        max_height = column.min_size().y.max(max_height);
    }

    // Make sure we fit everything next frame:
    //let total_required_width = total_spacing + max_column_width * (num_columns as f32);
    let total_required_width = total_width;

    let size = egui::vec2(ui.available_width().max(total_required_width), max_height);
    ui.allocate_rect(
        egui::Rect::from_min_size(top_left, size),
        egui::Sense {
            click: false,
            drag: false,
            focusable: false,
        },
    );
    result
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark()); // Switch to light mode
        let start = Instant::now();
        match self.snes.fetch_all(&mut self.client) {
            Err(e) => {
                println!("{}", e);
                frame.quit();
                return;
            }
            Ok(()) => {}
        }
        let elapsed = start.elapsed().as_millis();
        self.latency_samples.push_back(elapsed);
        if self.latency_samples.len() > 1000 {
            self.latency_samples.pop_front();
        }
        let average_latency: u128 =
            self.latency_samples.iter().sum::<u128>() / self.latency_samples.len() as u128;
        let mut s = 0;
        for x in self.latency_samples.iter() {
            let y = *x as i128;
            let avg = average_latency as i128;
            let diff = y - avg;
            s += diff * diff;
        }
        let stddev = (s as f64 / (self.latency_samples.len() as f64 - 1f64)).sqrt();
        if self.snes.start() {
            self.timer.start();
        }
        if self.snes.reset() {
            self.timer.reset(true);
        }
        if self.split() {
            self.timer.split();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            //egui::containers::Area::new("area").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(mk_text(
                    &format!(
                        "{} - {}",
                        self.timer.run().game_name(),
                        self.timer.run().category_name()
                    ),
                    42.0,
                    egui::Color32::WHITE,
                ));
                ui.label(mk_text(
                    &format!("latency {}ms ± {:02}ms", average_latency, stddev.round()),
                    10.0,
                    egui::Color32::GRAY,
                ));
            });
            let time =
                (self.timer.run().offset() + self.timer.current_attempt_duration()).to_duration();
            let ms_str = format!("{:02}", time.subsec_milliseconds());
            let time_str = format!(
                "{:02}:{:02}:{:02}.{ms:.*}",
                time.whole_hours(),
                time.whole_minutes() % 60,
                time.whole_seconds() % 60,
                2,
                ms = ms_str
            );
            let current_split_index = self.timer.current_split_index();
            let font_id = epaint::text::FontId {
                size: 32.0,
                family: epaint::text::FontFamily::Proportional,
            };
            let row_height = ui.fonts().row_height(&font_id);
            let segments = self.timer.run().segments();
            //let split_index = std::cmp::min(current_split_index.unwrap_or(0), segments.len() - 1);
            //ui.label(segments[split_index].name());
            let total_width = ui.available_width() - ui.spacing().item_spacing.x * 3.0;
            let other_col_width = 180.0;
            let split_col_width = total_width - 2.0 * other_col_width;
            //let split_col_width = 200.0;
            //let other_col_width = (1.0 - split_col_width) / 2.0;
            //let other_col_width = 0.165;
            columns(
                ui,
                &[
                    (
                        split_col_width,
                        egui::Layout::left_to_right().with_cross_align(egui::Align::Min),
                    ),
                    (
                        other_col_width,
                        egui::Layout::right_to_left().with_cross_align(egui::Align::Min),
                    ),
                    (
                        other_col_width,
                        egui::Layout::right_to_left().with_cross_align(egui::Align::Min),
                    ),
                ],
                |col| {
                    col[0].add(egui::Label::new(mk_text("", 32.0, egui::Color32::WHITE)));
                    col[1].add(egui::Label::new(mk_text("PB", 32.0, egui::Color32::WHITE)));
                    col[2].add(egui::Label::new(mk_text(
                        "Time",
                        32.0,
                        egui::Color32::WHITE,
                    )));
                },
            );
            ui.separator();
            ScrollArea::vertical()
                .min_scrolled_height(row_height)
                .max_height((row_height + ui.spacing().item_spacing.y) * 5.0)
                .show_viewport(ui, |ui, _viewport| {
                    for row_index in 0..segments.len() {
                        let this_row_is_highlighted = match current_split_index {
                            None => false,
                            Some(i) => i == row_index,
                        };
                        let row_time = match segments[row_index].split_time().real_time {
                            None => time,
                            Some(rt) => rt.to_duration(),
                        };
                        //let row_time = livesplit_core::TimeSpan::zero().to_duration();
                        let row_pb_time = segments[row_index].personal_best_split_time();
                        let ms_str = format!("{:02}", row_time.subsec_milliseconds());
                        let time_str = format!(
                            "{:02}:{:02}:{:02}.{ms:.*}",
                            row_time.whole_hours(),
                            row_time.whole_minutes() % 60,
                            row_time.whole_seconds() % 60,
                            2,
                            ms = ms_str
                        );
                        let frame = egui::Frame::none();
                        let frame = if this_row_is_highlighted {
                            frame.fill(egui::Color32::BLUE)
                        } else {
                            frame
                        };
                        frame.show(ui, |ui| {
                            columns(
                                ui,
                                &[
                                    (
                                        split_col_width,
                                        egui::Layout::left_to_right()
                                            .with_cross_align(egui::Align::Min),
                                    ),
                                    (
                                        other_col_width,
                                        egui::Layout::right_to_left()
                                            .with_cross_align(egui::Align::Min),
                                    ),
                                    (
                                        other_col_width,
                                        egui::Layout::right_to_left()
                                            .with_cross_align(egui::Align::Min),
                                    ),
                                ],
                                |col| {
                                    // Split name
                                    col[0].label(mk_text(
                                        segments[row_index].name(),
                                        32.0,
                                        egui::Color32::WHITE,
                                    ));
                                    // PB comparison
                                    col[1].scope(|ui| {
                                        match current_split_index {
                                            Some(i)
                                                if row_index < i
                                                    && segments[row_index]
                                                        .split_time()
                                                        .real_time
                                                        .is_some() =>
                                            {
                                                // show comparison
                                                match row_pb_time.real_time {
                                                    None => {
                                                        ui.label(mk_text(
                                                            "",
                                                            32.0,
                                                            egui::Color32::WHITE,
                                                        ));
                                                    }
                                                    Some(rt) => {
                                                        let diff = row_time - rt.to_duration();
                                                        ui.label(mk_text(
                                                            &format!("{}", diff),
                                                            32.0,
                                                            egui::Color32::WHITE,
                                                        ));
                                                    }
                                                };
                                            }
                                            _ => {
                                                ui.label(mk_text("", 32.0, egui::Color32::WHITE));
                                            }
                                        }
                                    });
                                    // Time
                                    col[2].scope(|ui| match current_split_index {
                                        Some(i) if i == row_index => {
                                            ui.label(mk_text(
                                                &time_str,
                                                32.0,
                                                egui::Color32::WHITE,
                                            ));
                                        }
                                        Some(i)
                                            if row_index < i
                                                && segments[row_index]
                                                    .split_time()
                                                    .real_time
                                                    .is_some() =>
                                        {
                                            ui.label(mk_text(
                                                &time_str,
                                                32.0,
                                                egui::Color32::WHITE,
                                            ));
                                        }
                                        _ => {
                                            ui.label(mk_text("", 32.0, egui::Color32::WHITE));
                                            //ui.label(mk_text(&time_str, 32.0, egui::Color32::WHITE));
                                        }
                                    });
                                },
                            );
                        });
                        if this_row_is_highlighted {
                            //ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                            ui.scroll_to_cursor(Some(egui::Align::Center));
                            //ui.scroll_to_cursor(None);
                        }
                    }
                });
            ui.separator();
            ui.with_layout(
                egui::Layout::right_to_left().with_cross_align(egui::Align::Min),
                |ui| {
                    ui.label(mk_text(&time_str, 42.0, egui::Color32::GREEN));
                },
            );
            self.remaining_space = ui.available_size_before_wrap();
        });
        ctx.request_repaint();
        let mut sz = ctx.input().screen_rect.size();
        sz.y -= self.remaining_space.y;
        frame.set_window_size(sz);
        //println!("sz = {:#?}", sz);
    }
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let mut client = usb2snes::usb2snes::SyncClient::connect();
    client.set_name("annelid".to_owned())?;
    println!("Server version is {:?}", client.app_version());
    let mut devices = client.list_device()?;
    if devices.len() != 1 {
        if devices.len() < 1 {
            Err("No devices present")?;
        } else {
            Err(format!("You need to select a device: {:#?}", devices))?;
        }
    }
    let device = devices.pop().ok_or("Device list was empty")?;
    println!("Using device: {}", device);
    client.attach(&device)?;
    println!("Connected.");
    println!("{:#?}", client.info()?);
    let options = eframe::NativeOptions {
        always_on_top: true,
        // TODO: fix me
        initial_window_size: Some(egui::vec2(470.0, 337.0)),
        ..eframe::NativeOptions::default()
    };
    println!("size = {:#?}", options.initial_window_size);
    let mut snes = SNESState::new();
    // We need to initialize the memory state before entering the polling loop
    snes.fetch_all(&mut client)?;

    let (settings, run) = hundo();
    //let (settings, run) = anypercent();

    let app = MyApp {
        client: client,
        snes: snes,
        settings: settings,
        pickedUpHundredthMissile: false,
        pickedUpSporeSpawnSuper: false,
        latency_samples: VecDeque::from([]),
        timer: Timer::new(run).expect("Run with at least one segment provided"),
        remaining_space: egui::Vec2 { x: 0.0, y: 0.0 },
    };
    eframe::run_native("Annelid", options, Box::new(|_cc| Box::new(app)));
}

fn hundo() -> (Settings, livesplit_core::Run) {
    let mut settings = Settings::new();
    settings.split_on_hundo();
    let mut run = Run::new();
    run.set_game_name("Super Metroid");
    run.set_category_name("100%");
    run.push_segment(Segment::new("ceresRidley"));
    run.push_segment(Segment::new("ceresEscape"));
    run.push_segment(Segment::new("morphBall"));
    run.push_segment(Segment::new("firstMissile"));
    run.push_segment(Segment::new("bomb"));
    run.push_segment(Segment::new("bomb torizo"));
    run.push_segment(Segment::new("terminator tank"));
    run.push_segment(Segment::new("early supers"));
    run.push_segment(Segment::new("missile behind reserve"));
    run.push_segment(Segment::new("missile behind behind"));
    run.push_segment(Segment::new("brin reserve"));
    run.push_segment(Segment::new("missile quick fall"));
    run.push_segment(Segment::new("charge beam"));
    run.push_segment(Segment::new("spazer"));
    run.push_segment(Segment::new("kraid"));
    run.push_segment(Segment::new("varia"));
    run.push_segment(Segment::new("beetom tank"));
    run.push_segment(Segment::new("hjb tank"));
    run.push_segment(Segment::new("boots"));
    run.push_segment(Segment::new("missile boots"));
    run.push_segment(Segment::new("missile cathedral"));
    run.push_segment(Segment::new("missile speedbooster"));
    run.push_segment(Segment::new("speedbooster"));
    run.push_segment(Segment::new("missile wave"));
    run.push_segment(Segment::new("wave"));
    run.push_segment(Segment::new("croc tank"));
    run.push_segment(Segment::new("crocomire"));
    run.push_segment(Segment::new("croc pb"));
    run.push_segment(Segment::new("grapple"));
    run.push_segment(Segment::new("missile grapple"));
    run.push_segment(Segment::new("missile swag dboost"));
    run.push_segment(Segment::new("missile croc escape"));
    run.push_segment(Segment::new("missile alpha"));
    run.push_segment(Segment::new("alpha pb"));
    run.push_segment(Segment::new("beta pb"));
    run.push_segment(Segment::new("missile moat"));
    run.push_segment(Segment::new("missile spooky"));
    run.push_segment(Segment::new("phantoon"));
    run.push_segment(Segment::new("right super"));
    run.push_segment(Segment::new("left super"));
    run.push_segment(Segment::new("ws tank"));
    run.push_segment(Segment::new("missile attic"));
    run.push_segment(Segment::new("missile sky"));
    run.push_segment(Segment::new("missile tunnel"));
    run.push_segment(Segment::new("missile bowling"));
    run.push_segment(Segment::new("ws reserve"));
    run.push_segment(Segment::new("gravity"));
    run.push_segment(Segment::new("missile mermaid"));
    run.push_segment(Segment::new("criteria pb"));
    run.push_segment(Segment::new("gauntlet tank"));
    run.push_segment(Segment::new("gauntlet missile1"));
    run.push_segment(Segment::new("gauntlet missile2"));
    run.push_segment(Segment::new("etecoons tank"));
    run.push_segment(Segment::new("etecoons supers"));
    run.push_segment(Segment::new("etecoons pb"));
    run.push_segment(Segment::new("missile pink brin top"));
    run.push_segment(Segment::new("mission impossible pb"));
    run.push_segment(Segment::new("wave gate tank"));
    run.push_segment(Segment::new("spospo super"));
    run.push_segment(Segment::new("missile pink brin bottom"));
    run.push_segment(Segment::new("waterways tank"));
    run.push_segment(Segment::new("missile greenhills"));
    run.push_segment(Segment::new("maridia tube"));
    run.push_segment(Segment::new("missile mainstreet"));
    run.push_segment(Segment::new("momma turtle tank"));
    run.push_segment(Segment::new("missile momma turtle"));
    run.push_segment(Segment::new("crab supers"));
    run.push_segment(Segment::new("missile beach"));
    run.push_segment(Segment::new("wateringhole supers"));
    run.push_segment(Segment::new("missile wateringhole"));
    run.push_segment(Segment::new("botwoon"));
    run.push_segment(Segment::new("botwoon tank"));
    run.push_segment(Segment::new("missile precious room"));
    run.push_segment(Segment::new("draygon"));
    run.push_segment(Segment::new("spacejump"));
    run.push_segment(Segment::new("missile right sandpit"));
    run.push_segment(Segment::new("right sandpit pb"));
    run.push_segment(Segment::new("springball"));
    run.push_segment(Segment::new("plasma"));
    run.push_segment(Segment::new("missile aqueduct"));
    run.push_segment(Segment::new("aqueduct super"));
    run.push_segment(Segment::new("maridia reserve"));
    run.push_segment(Segment::new("missile left sandpit"));
    run.push_segment(Segment::new("missile wharehouse"));
    run.push_segment(Segment::new("ice"));
    run.push_segment(Segment::new("missile crumble shaft"));
    run.push_segment(Segment::new("missile gt"));
    run.push_segment(Segment::new("gt super"));
    run.push_segment(Segment::new("gt"));
    run.push_segment(Segment::new("screwattack"));
    run.push_segment(Segment::new("missile mickeymouse"));
    run.push_segment(Segment::new("pbs of shame"));
    run.push_segment(Segment::new("ridley"));
    run.push_segment(Segment::new("ridley tank"));
    run.push_segment(Segment::new("firefleas tank"));
    run.push_segment(Segment::new("missile hota"));
    run.push_segment(Segment::new("jail pb"));
    run.push_segment(Segment::new("missile ffz"));
    run.push_segment(Segment::new("missile norfair reserve"));
    run.push_segment(Segment::new("missile norfair reserve"));
    run.push_segment(Segment::new("norfair reserve"));
    run.push_segment(Segment::new("missile bubble mountain"));
    run.push_segment(Segment::new("xr_tech"));
    run.push_segment(Segment::new("retro pb"));
    run.push_segment(Segment::new("brin ceiling tank"));
    run.push_segment(Segment::new("missile billy mays 1"));
    run.push_segment(Segment::new("missile billy mays 2"));
    run.push_segment(Segment::new("missile retro brin"));
    run.push_segment(Segment::new("missile old mb"));
    run.push_segment(Segment::new("climb supers"));
    run.push_segment(Segment::new("missile dental plan"));
    run.push_segment(Segment::new("g4 room"));
    run.push_segment(Segment::new("the baby!"));
    run.push_segment(Segment::new("mb1"));
    run.push_segment(Segment::new("mb2"));
    run.push_segment(Segment::new("mb3"));
    run.push_segment(Segment::new(".done"));
    (settings, run)
}

fn anypercent() -> (Settings, livesplit_core::Run) {
    let mut settings = Settings::new();
    settings.split_on_anypercent();
    let mut run = Run::new();
    run.set_game_name("Super Metroid");
    run.set_category_name("KPDR");
    run.push_segment(Segment::new("ceresRidley"));
    run.push_segment(Segment::new("ceresEscape"));
    run.push_segment(Segment::new("morphBall"));
    run.push_segment(Segment::new("firstMissile"));
    run.push_segment(Segment::new("bomb"));
    run.push_segment(Segment::new("bomb torizo"));
    run.push_segment(Segment::new("terminator tank"));
    run.push_segment(Segment::new("early supers"));
    run.push_segment(Segment::new("charge missile"));
    run.push_segment(Segment::new("charge beam"));
    run.push_segment(Segment::new("kraid"));
    run.push_segment(Segment::new("varia"));
    run.push_segment(Segment::new("hjb tank"));
    run.push_segment(Segment::new("boots"));
    run.push_segment(Segment::new("speedbooster"));
    run.push_segment(Segment::new("missile wave"));
    run.push_segment(Segment::new("wave"));
    run.push_segment(Segment::new("alpha pb"));
    run.push_segment(Segment::new("phantoon"));
    run.push_segment(Segment::new("left super"));
    run.push_segment(Segment::new("gravity"));
    run.push_segment(Segment::new("maridia tube"));
    run.push_segment(Segment::new("botwoon"));
    run.push_segment(Segment::new("botwoon tank"));
    run.push_segment(Segment::new("draygon"));
    run.push_segment(Segment::new("spacejump"));
    run.push_segment(Segment::new("plasma"));
    run.push_segment(Segment::new("ice"));
    run.push_segment(Segment::new("ridley"));
    run.push_segment(Segment::new("g4 room"));
    run.push_segment(Segment::new("the baby!"));
    run.push_segment(Segment::new("mb1"));
    run.push_segment(Segment::new("mb2"));
    run.push_segment(Segment::new("mb3"));
    run.push_segment(Segment::new(".done"));
    (settings, run)
}
