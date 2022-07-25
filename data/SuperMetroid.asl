// Super Metroid autosplitter, hosted at:
// https://github.com/UNHchabo/AutoSplitters
// 
// Basic format of the script is based on:
// https://github.com/Spiraster/ASLScripts/tree/master/LiveSplit.SMW
// 
// Most of the RAM values taken from:
// https://jathys.zophar.net/supermetroid/kejardon/RAMMap.txt

state("higan"){}
state("bsnes"){}
state("snes9x"){}
state("snes9x-x64"){}
state("emuhawk"){}
state("retroarch"){}
state("lsnes-bsnes"){}

startup
{
    settings.Add("ammoPickups", true, "Ammo Pickups");
    settings.SetToolTip("ammoPickups", "Split on Missiles, Super Missiles, and Power Bombs");
    settings.Add("firstMissile", false, "First Missiles", "ammoPickups");
    settings.SetToolTip("firstMissile", "Split on the first Missile pickup");
    settings.Add("allMissiles", false, "All Missiles", "ammoPickups");
    settings.SetToolTip("allMissiles", "Split on each Missile upgrade");
    settings.Add("specificMissiles", false, "Specific Missile Packs", "ammoPickups");
    settings.SetToolTip("specificMissiles", "Split on specific Missile Pack locations");
    settings.Add("crateriaMissiles", false, "Crateria Missile Packs", "specificMissiles");
    settings.SetToolTip("crateriaMissiles", "Split on Crateria Missile Pack locations");
    settings.Add("oceanBottomMissiles", false, "Ocean Bottom Missile Pack", "crateriaMissiles");
    settings.SetToolTip("oceanBottomMissiles", "Split on picking up the Missile Pack located at the bottom left of the West Ocean");
    settings.Add("oceanTopMissiles", false, "Ocean Top Missile Pack", "crateriaMissiles");
    settings.SetToolTip("oceanTopMissiles", "Split on picking up the Missile Pack located in the ceiling tile in West Ocean");
    settings.Add("oceanMiddleMissiles", false, "Ocean Middle Missile Pack", "crateriaMissiles");
    settings.SetToolTip("oceanMiddleMissiles", "Split on picking up the Missile Pack located in the Morphball maze section of West Ocean");
    settings.Add("moatMissiles", false, "Moat Missile Pack", "crateriaMissiles");
    settings.SetToolTip("moatMissiles", "Split on picking up the Missile Pack in The Moat, also known as The Lake");
    settings.Add("oldTourianMissiles", false, "Old Tourian Missile Pack", "crateriaMissiles");
    settings.SetToolTip("oldTourianMissiles", "Split on picking up the Missile Pack in the Pit Room");
    settings.Add("gauntletRightMissiles", false, "Gauntlet Right Missile Pack", "crateriaMissiles");
    settings.SetToolTip("gauntletRightMissiles", "Split on picking up the right side Missile Pack at the end of Gauntlet(Green Pirates Shaft)");
    settings.Add("gauntletLeftMissiles", false, "Gauntlet Left Missile Pack", "crateriaMissiles");
    settings.SetToolTip("gauntletLeftMissiles", "Split on picking up the left side Missile Pack at the end of Gauntlet(Green Pirates Shaft)");
    settings.Add("dentalPlan", false, "Dental Plan Missile Pack", "crateriaMissiles");
    settings.SetToolTip("dentalPlan", "Split on picking up the Missile Pack located in The Final Missile");
    settings.Add("brinstarMissiles", false, "Brinstar Missile Packs", "specificMissiles");
    settings.SetToolTip("brinstarMissiles", "Split on Brinstar Missile Pack locations");
    settings.Add("earlySuperBridgeMissiles", false, "Early Supers Under Bridge Missile Pack", "brinstarMissiles");
    settings.SetToolTip("earlySuperBridgeMissiles", "Split on picking up the Missile Pack located below the crumble bridge in the Early Supers Room");
    settings.Add("greenBrinstarReserveMissiles", false, "Brinstar Reserve Missile Pack", "brinstarMissiles");
    settings.SetToolTip("greenBrinstarReserveMissiles", "Split on picking up the first Missile Pack behind the Brinstar Reserve Tank");
    settings.Add("greenBrinstarExtraReserveMissiles", false, "Brinstar Reserve Missile Pack 2", "brinstarMissiles");
    settings.SetToolTip("greenBrinstarExtraReserveMissiles", "Split on picking up the second Missile Pack behind the Brinstar Reserve Tank Room");
    settings.Add("bigPinkTopMissiles", false, "Big Pink Top Missile Pack", "brinstarMissiles");
    settings.SetToolTip("bigPinkTopMissiles", "Split on picking up the Missile Pack located left of center in Big Pink");
    settings.Add("chargeMissiles", false, "Charge Missile Pack", "brinstarMissiles");
    settings.SetToolTip("chargeMissiles", "Split on picking up the Missile Pack located at the bottom left of Big Pink");
    settings.Add("greenHillsMissiles", false, "Green Hills Missile Pack", "brinstarMissiles");
    settings.SetToolTip("greenHillsMissiles", "Split on picking up the Missile Pack in Green Hill Zone");
    settings.Add("blueBrinstarETankMissiles", false, "Classic Brinstar East Missile Pack", "brinstarMissiles");
    settings.SetToolTip("blueBrinstarETankMissiles", "Split on picking up the Missile Pack in the Blue Brinstar Energy Tank Room");
    settings.Add("alphaMissiles", false, "Alpha Missile Pack", "brinstarMissiles");
    settings.SetToolTip("alphaMissiles", "Split on picking up the first Missile Pack of the game(First Missile Room)");
    settings.Add("billyMaysMissiles", false, "Billy Mays Missile Pack", "brinstarMissiles");
    settings.SetToolTip("billyMaysMissiles", "Split on picking up the Missile Pack located on the pedestal in Billy Mays' Room");
    settings.Add("butWaitTheresMoreMissiles", false, "But Wait... There's MORE! Missile Pack", "brinstarMissiles");
    settings.SetToolTip("butWaitTheresMoreMissiles", "Split on picking up the Missile Pack located in the floor of Billy Mays' Room");
    settings.Add("redBrinstarMissiles", false, "Red Brinstar Missile Pack", "brinstarMissiles");
    settings.SetToolTip("redBrinstarMissiles", "Split on picking up the Missile Pack in the Alpha Power Bombs Room");
    settings.Add("warehouseMissiles", false, "Warehouse Missile Pack", "brinstarMissiles");
    settings.SetToolTip("warehouseMissiles", "Split on picking up the Missile Pack in the Warehouse Kihunter Room");
    settings.Add("norfairMissiles", false, "Norfair Missile Packs", "specificMissiles");
    settings.SetToolTip("norfairMissiles", "Split on Norfair Missile Pack locations");
    settings.Add("cathedralMissiles", false, "Cathedral Missile Pack", "norfairMissiles");
    settings.SetToolTip("cathedralMissiles", "Split on picking up the Missile Pack in Cathedral");
    settings.Add("crumbleShaftMissiles", false, "Crumble Shaft Missile Pack", "norfairMissiles");
    settings.SetToolTip("crumbleShaftMissiles", "Split on picking up the Missile Pack in Crumble Shaft");
    settings.Add("crocomireEscapeMissiles", false, "Crocomire Escape Missile Pack", "norfairMissiles");
    settings.SetToolTip("crocomireEscapeMissiles", "Split on picking up the Missile Pack in Crocomire Escape");
    settings.Add("hiJumpMissiles", false, "Hi Jump Missile Pack", "norfairMissiles");
    settings.SetToolTip("hiJumpMissiles", "Split on picking up the Missile Pack in the Hi Jump Energy Tank Room");
    settings.Add("postCrocomireMissiles", false, "Post Crocomire Missile Pack", "norfairMissiles");
    settings.SetToolTip("postCrocomireMissiles", "Split on picking up the Missile Pack in the Post Crocomire Missile Room, also known as Cosine Room");
    settings.Add("grappleMissiles", false, "Grapple Missile Pack", "norfairMissiles");
    settings.SetToolTip("grappleMissiles", "Split on picking up the Missile Pack in the Post Crocomire Jump Room");
    settings.Add("norfairReserveMissiles", false, "Norfair Reserve Missile Pack", "norfairMissiles");
    settings.SetToolTip("norfairReserveMissiles", "Split on picking up the Missile Pack in the Norfair Reserve Tank Room");
    settings.Add("greenBubblesMissiles", false, "Green Bubbles Missile Pack", "norfairMissiles");
    settings.SetToolTip("greenBubblesMissiles", "Split on picking up the Missile Pack in the Green Bubbles Missile Room");
    settings.Add("bubbleMountainMissiles", false, "Bubble Mountain Missile Pack", "norfairMissiles");
    settings.SetToolTip("bubbleMountainMissiles", "Split on picking up the Missile Pack in Bubble Mountain");
    settings.Add("speedBoostMissiles", false, "Speed Booster Missile Pack", "norfairMissiles");
    settings.SetToolTip("speedBoostMissiles", "Split on picking up the Missile Pack in Speed Booster Hall");
    settings.Add("waveMissiles", false, "Wave Beam Missile Pack", "norfairMissiles");
    settings.SetToolTip("waveMissiles", "Split on picking up the Wave Missile Pack in Double Chamber");
    settings.Add("goldTorizoMissiles", false, "Golden Torizo Missile Pack", "norfairMissiles");
    settings.SetToolTip("goldTorizoMissiles", "Split on picking up the Missile Pack in the Golden Torizo's Room");
    settings.Add("mickeyMouseMissiles", false, "Mickey Mouse Missile Pack", "norfairMissiles");
    settings.SetToolTip("mickeyMouseMissiles", "Split on picking up the Missile Pack in the Mickey Mouse Room");
    settings.Add("lowerNorfairSpringMazeMissiles", false, "Lower Norfair Springball Maze Missile Pack", "norfairMissiles");
    settings.SetToolTip("lowerNorfairSpringMazeMissiles", "Split on picking up the Missile Pack in the Lower Norfair Springball Maze Room");
    settings.Add("threeMusketeersMissiles", false, "Three Musketeers Missile Pack", "norfairMissiles");
    settings.SetToolTip("threeMusketeersMissiles", "Split on picking up the Missile Pack in the The Musketeers' Room");
    settings.Add("wreckedShipMissiles", false, "Wrecked Ship Missile Packs", "specificMissiles");
    settings.SetToolTip("wreckedShipMissiles", "Split on Wrecked Ship Missile Pack locations");
    settings.Add("wreckedShipMainShaftMissiles", false, "Wrecked Ship Main Shaft Missile Pack", "wreckedShipMissiles");
    settings.SetToolTip("wreckedShipMainShaftMissiles", "Split on picking up the Missile Pack in Wrecked Ship Main Shaft");
    settings.Add("bowlingMissiles", false, "Bowling Alley Missile Pack", "wreckedShipMissiles");
    settings.SetToolTip("bowlingMissiles", "Split on picking up the Missile Pack in Bowling Alley");
    settings.Add("atticMissiles", false, "Attic Missile Pack", "wreckedShipMissiles");
    settings.SetToolTip("atticMissiles", "Split on picking up the Missile Pack in the Wrecked Ship East Missile Room");
    settings.Add("maridiaMissiles", false, "Maridia Missile Packs", "specificMissiles");
    settings.SetToolTip("maridiaMissiles", "Split on Maridia Missile Pack locations");
    settings.Add("mainStreetMissiles", false, "Main Street Missile Pack", "maridiaMissiles");
    settings.SetToolTip("mainStreetMissiles", "Split on picking up the Missile Pack in Main Street");
    settings.Add("mamaTurtleMissiles", false, "Mama Turtle Missile Pack", "maridiaMissiles");
    settings.SetToolTip("mamaTurtleMissiles", "Split on picking up the Missile Pack in the Mama Turtle Room");
    settings.Add("wateringHoleMissiles", false, "Watering Hole Missile Pack", "maridiaMissiles");
    settings.SetToolTip("wateringHoleMissiles", "Split on picking up the Missile Pack in Watering Hole");
    settings.Add("beachMissiles", false, "Beach Missile Pack", "maridiaMissiles");
    settings.SetToolTip("beachMissiles", "Split on picking up the Missile Pack in the Pseudo Plasma Spark Room");
    settings.Add("leftSandPitMissiles", false, "Left Sand Pit Missile Pack", "maridiaMissiles");
    settings.SetToolTip("leftSandPitMissiles", "Split on picking up the Missile Pack in West Sand Hole");
    settings.Add("rightSandPitMissiles", false, "Right Sand Pit Missile Pack", "maridiaMissiles");
    settings.SetToolTip("rightSandPitMissiles", "Split on picking up the Missile Pack in East Sand Hole");
    settings.Add("aqueductMissiles", false, "Aqueduct Missile Pack", "maridiaMissiles");
    settings.SetToolTip("aqueductMissiles", "Split on picking up the Missile Pack in Aqueduct");
    settings.Add("preDraygonMissiles", false, "Pre Draygon Missile Pack", "maridiaMissiles");
    settings.SetToolTip("preDraygonMissiles", "Split on picking up the Missile Pack in The Precious Room");
    settings.Add("firstSuper", false, "First Supers", "ammoPickups");
    settings.SetToolTip("firstSuper", "Split on the first Super Missile pickup");
    settings.Add("allSupers", false, "All Super Missiles", "ammoPickups");
    settings.SetToolTip("allSupers", "Split on each Super Missile upgrade");
    settings.Add("specificSupers", false, "Specific Super Missile Packs", "ammoPickups");
    settings.SetToolTip("specificSupers", "Split on specific Super Missile Pack locations");
    settings.Add("climbSupers", false, "Crateria Super Missile Pack", "specificSupers");
    settings.SetToolTip("climbSupers", "Split on picking up the Super Missile Pack in the Crateria Super Room");
    settings.Add("sporeSpawnSupers", false, "Spore Spawn Super Missile Pack", "specificSupers");
    settings.SetToolTip("sporeSpawnSupers", "Split on picking up the Super Missile Pack in the Spore Spawn Super Room (NOTE: SSTRA splits when the dialogue box disappears, not on touch. Use Spore Spawn RTA Finish for SSTRA runs.)");
    settings.Add("earlySupers", false, "Early Super Missile Pack", "specificSupers");
    settings.SetToolTip("earlySupers", "Split on picking up the Super Missile Pack in the Early Supers Room");
    settings.Add("etacoonSupers", false, "Etacoon Super Missile Pack", "specificSupers");
    settings.SetToolTip("etacoonSupers", "Split on picking up the Super Missile Pack in the Etacoon Super Room");
    settings.Add("goldTorizoSupers", false, "Golden Torizo Super Missile Pack", "specificSupers");
    settings.SetToolTip("goldTorizoSupers", "Split on picking up the Super Missile Pack in the Golden Torizo's Room");
    settings.Add("wreckedShipLeftSupers", false, "Wrecked Ship Left Super Missile Pack", "specificSupers");
    settings.SetToolTip("wreckedShipLeftSupers", "Split on picking up the Super Missile Pack in the Wrecked Ship West Super Room");
    settings.Add("wreckedShipRightSupers", false, "Wrecked Ship Right Super Missile Pack", "specificSupers");
    settings.SetToolTip("wreckedShipRightSupers", "Split on picking up the Super Missile Pack in the Wrecked Ship East Super Room");
    settings.Add("crabSupers", false, "Crab Super Missile Pack", "specificSupers");
    settings.SetToolTip("crabSupers", "Split on picking up the Super Missile Pack in Main Street");
    settings.Add("wateringHoleSupers", false, "Watering Hole Super Missile Pack", "specificSupers");
    settings.SetToolTip("wateringHoleSupers", "Split on picking up the Super Missile Pack in Watering Hole");
    settings.Add("aqueductSupers", false, "Aqueduct Super Missile Pack", "specificSupers");
    settings.SetToolTip("aqueductSupers", "Split on picking up the Super Missile Pack in Aqueduct");
    settings.Add("firstPowerBomb", true, "First Power Bomb", "ammoPickups");
    settings.SetToolTip("firstPowerBomb", "Split on the first Power Bomb pickup");
    settings.Add("allPowerBombs", false, "All Power Bombs", "ammoPickups");
    settings.SetToolTip("allPowerBombs", "Split on each Power Bomb upgrade");
    settings.Add("specificBombs", false, "Specific Power Bomb Packs", "ammoPickups");
    settings.SetToolTip("specificBombs", "Split on specific Power Bomb Pack locations");
    settings.Add("landingSiteBombs", false, "Crateria Power Bomb Pack", "specificBombs");
    settings.SetToolTip("landingSiteBombs", "Split on picking up the Power Bomb Pack in the Crateria Power Bomb Room");
    settings.Add("etacoonBombs", false, "Etacoon Power Bomb Pack", "specificBombs");
    settings.SetToolTip("etacoonBombs", "Split on picking up the Power Bomb Pack in the Etacoon Room section of Green Brinstar Main Shaft");
    settings.Add("pinkBrinstarBombs", false, "Pink Brinstar Power Bomb Pack", "specificBombs");
    settings.SetToolTip("pinkBrinstarBombs", "Split on picking up the Power Bomb Pack in the Pink Brinstar Power Bomb Room");
    settings.Add("blueBrinstarBombs", false, "Classic Brinstar Power Bomb Pack", "specificBombs");
    settings.SetToolTip("blueBrinstarBombs", "Split on picking up the Power Bomb Pack in the Morph Ball Room");
    settings.Add("alphaBombs", false, "Alpha Power Bomb Pack", "specificBombs");
    settings.SetToolTip("alphaBombs", "Split on picking up the Power Bomb Pack in the Alpha Power Bomb Room");
    settings.Add("betaBombs", false, "Beta Power Bomb Pack", "specificBombs");
    settings.SetToolTip("betaBombs", "Split on picking up the Power Bomb Pack in the Beta Power Bomb Room");
    settings.Add("crocomireBombs", false, "Crocomire Power Bomb Pack", "specificBombs");
    settings.SetToolTip("crocomireBombs", "Split on picking up the Power Bomb Pack in the Post Crocomire Power Bomb Room");
    settings.Add("lowerNorfairEscapeBombs", false, "Lower Norfair Escape Power Bomb Pack", "specificBombs");
    settings.SetToolTip("lowerNorfairEscapeBombs", "Split on picking up the Power Bomb Pack in the Lower Norfair Escape Power Bomb Room");
    settings.Add("shameBombs", false, "Power Bombs of Shame Pack", "specificBombs");
    settings.SetToolTip("shameBombs", "Split on picking up the Power Bomb Pack in Wasteland");
    settings.Add("rightSandPitBombs", false, "Maridia Power Bomb Pack", "specificBombs");
    settings.SetToolTip("rightSandPitBombs", "Split on picking up the Power Bomb Pack in East Sand Hall");

    settings.Add("suitUpgrades", true, "Suit Pickups");
    settings.SetToolTip("suitUpgrades", "Split on Varia and Gravity pickups");
    settings.Add("variaSuit", true, "Varia Suit", "suitUpgrades");
    settings.SetToolTip("variaSuit", "Split on picking up the Varia Suit");
    settings.Add("gravSuit", true, "Gravity Suit", "suitUpgrades");
    settings.SetToolTip("gravSuit", "Split on picking up the Gravity Suit");

    settings.Add("beamUpgrades", true, "Beam Upgrades");
    settings.SetToolTip("beamUpgrades", "Split on beam upgrades");
    settings.Add("chargeBeam", false, "Charge Beam", "beamUpgrades");
    settings.SetToolTip("chargeBeam", "Split on picking up the Charge Beam");
    settings.Add("spazer", false, "Spazer", "beamUpgrades");
    settings.SetToolTip("spazer", "Split on picking up the Spazer");
    settings.Add("wave", true, "Wave Beam", "beamUpgrades");
    settings.SetToolTip("wave", "Split on picking up the Wave Beam");
    settings.Add("ice", false, "Ice Beam", "beamUpgrades");
    settings.SetToolTip("ice", "Split on picking up the Ice Beam");
    settings.Add("plasma", false, "Plasma Beam", "beamUpgrades");
    settings.SetToolTip("plasma", "Split on picking up the Plasma Beam");

    settings.Add("bootUpgrades", false, "Boot Upgrades");
    settings.SetToolTip("bootUpgrades", "Split on boot upgrades");
    settings.Add("hiJump", false, "Hi-Jump Boots", "bootUpgrades");
    settings.SetToolTip("hiJump", "Split on picking up the Hi-Jump Boots");
    settings.Add("spaceJump", false, "Space Jump", "bootUpgrades");
    settings.SetToolTip("spaceJump", "Split on picking up Space Jump");
    settings.Add("speedBooster", false, "Speed Booster", "bootUpgrades");
    settings.SetToolTip("speedBooster", "Split on picking up the Speed Booster");

    settings.Add("energyUpgrades", false, "Energy Upgrades");
    settings.SetToolTip("energyUpgrades", "Split on Energy Tanks and Reserve Tanks");
    settings.Add("firstETank", false, "First Energy Tank", "energyUpgrades");
    settings.SetToolTip("firstETank", "Split on picking up the first Energy Tank");
    settings.Add("allETanks", false, "All Energy Tanks", "energyUpgrades");
    settings.SetToolTip("allETanks", "Split on picking up each Energy Tank");
    settings.Add("specificETanks", false, "Specific Energy Tanks", "energyUpgrades");
    settings.SetToolTip("specificETanks", "Split on specific Energy Tank locations");
    settings.Add("gauntletETank", false, "Gauntlet Energy Tank", "specificETanks");
    settings.SetToolTip("gauntletETank", "Split on picking up the Energy Tank in the Gauntlet Energy Tank Room");
    settings.Add("terminatorETank", false, "Terminator Energy Tank", "specificETanks");
    settings.SetToolTip("terminatorETank", "Split on picking up the Energy Tank in the Terminator Room");
    settings.Add("ceilingETank", false, "Classic Brinstar Energy Tank", "specificETanks");
    settings.SetToolTip("ceilingETank", "Split on picking up the Energy Tank in the Blue Brinstar Energy Tank Room");
    settings.Add("etecoonsETank", false, "Etacoon Energy Tank", "specificETanks");
    settings.SetToolTip("etecoonsETank", "Split on picking up the Energy Tank in the Etacoon Energy Tank Room");
    settings.Add("waterwayETank", false, "Waterway Energy Tank", "specificETanks");
    settings.SetToolTip("waterwayETank", "Split on picking up the Energy Tank in Waterway");
    settings.Add("waveGateETank", false, "Pink Brinstar Wave Gate Energy Tank", "specificETanks");
    settings.SetToolTip("waveGateETank", "Split on picking up the Energy Tank in the Hopper Energy Tank Room");
    settings.Add("kraidETank", false, "Warehouse Energy Tank", "specificETanks");
    settings.SetToolTip("kraidETank", "Split on picking up the Kraid Energy Tank in the Warehouse Energy Tank Room");
    settings.Add("crocomireETank", false, "Crocomire Energy Tank", "specificETanks");
    settings.SetToolTip("crocomireETank", "Split on picking up the Energy Tank in Crocomire's Room");
    settings.Add("hiJumpETank", false, "Hi Jump Energy Tank", "specificETanks");
    settings.SetToolTip("hiJumpETank", "Split on picking up the Energy Tank in the Hi Jump Energy Tank Room");
    settings.Add("ridleyETank", false, "Ridley Energy Tank", "specificETanks");
    settings.SetToolTip("ridleyETank", "Split on picking up the Energy Tank in the Ridley Tank Room");
    settings.Add("firefleaETank", false, "Fireflea Energy Tank", "specificETanks");
    settings.SetToolTip("firefleaETank", "Split on picking up the Energy Tank in the Lower Norfair Fireflea Room");
    settings.Add("wreckedShipETank", false, "Wrecked Ship Energy Tank", "specificETanks");
    settings.SetToolTip("wreckedShipETank", "Split on picking up the Energy Tank in the Wrecked Ship Energy Tank Room");
    settings.Add("tatoriETank", false, "Mama Turtle Energy Tank", "specificETanks");
    settings.SetToolTip("tatoriETank", "Split on picking up the Energy Tank in the Mama Turtle Room");
    settings.Add("botwoonETank", false, "Botwoon Energy Tank", "specificETanks");
    settings.SetToolTip("botwoonETank", "Split on picking up the Energy Tank in the Botwoon Energy Tank Room");
    settings.Add("reserveTanks", false, "All Reserve Tanks", "energyUpgrades");
    settings.SetToolTip("reserveTanks", "Split on picking up each Reserve Tank");
    settings.Add("specificRTanks", false, "Specific Reserve Tanks", "energyUpgrades");
    settings.SetToolTip("specificRTanks", "Split on specific Reserve Tank locations");
    settings.Add("brinstarReserve", false, "Brinstar Reserve Tank", "specificRTanks");
    settings.SetToolTip("brinstarReserve", "Split on picking up the Reserve Tank in the Brinstar Reserve Tank Room");
    settings.Add("norfairReserve", false, "Norfair Reserve Tank", "specificRTanks");
    settings.SetToolTip("norfairReserve", "Split on picking up the Reserve Tank in the Norfair Reserve Tank Room");
    settings.Add("wreckedShipReserve", false, "Wrecked Ship Reserve Tank", "specificRTanks");
    settings.SetToolTip("wreckedShipReserve", "Split on picking up the Reserve Tank in Bowling Alley");
    settings.Add("maridiaReserve", false, "Maridia Reserve Tank", "specificRTanks");
    settings.SetToolTip("maridiaReserve", "Split on picking up the Reserve Tank in West Sand Hole");

    settings.Add("miscUpgrades", false, "Misc Upgrades");
    settings.SetToolTip("miscUpgrades", "Split on the miscellaneous upgrades");
    settings.Add("morphBall", false, "Morphing Ball", "miscUpgrades");
    settings.SetToolTip("morphBall", "Split on picking up the Morphing Ball");
    settings.Add("bomb", false, "Bomb",  "miscUpgrades");
    settings.SetToolTip("bomb", "Split on picking up the Bomb");
    settings.Add("springBall", false, "Spring Ball", "miscUpgrades");
    settings.SetToolTip("springBall", "Split on picking up the Spring Ball");
    settings.Add("screwAttack", false, "Screw Attack", "miscUpgrades");
    settings.SetToolTip("screwAttack", "Split on picking up the Screw Attack");
    settings.Add("grapple", false, "Grapple Beam", "miscUpgrades");
    settings.SetToolTip("grapple", "Split on picking up the Grapple Beam");
    settings.Add("xray", false, "X-Ray Scope", "miscUpgrades");
    settings.SetToolTip("xray", "Split on picking up the X-Ray Scope");

    settings.Add("areaTransitions", true, "Area Transitions");
    settings.SetToolTip("areaTransitions", "Split on transitions between areas");
    settings.Add("miniBossRooms", false, "Miniboss Rooms", "areaTransitions");
    settings.SetToolTip("miniBossRooms", "Split on entering miniboss rooms (except Bomb Torizo)");
    settings.Add("bossRooms", false, "Boss Rooms", "areaTransitions");
    settings.SetToolTip("bossRooms", "Split on entering major boss rooms");
    settings.Add("elevatorTransitions", false, "Elevator transitions", "areaTransitions");
    settings.SetToolTip("elevatorTransitions", "Split on elevator transitions between areas (except Statue Room to Tourian)");
    settings.Add("ceresEscape", false, "Ceres Escape", "areaTransitions");
    settings.SetToolTip("ceresEscape", "Split on leaving Ceres Station");
    settings.Add("wreckedShipEntrance", false, "Wrecked Ship Entrance", "areaTransitions");
    settings.SetToolTip("wreckedShipEntrance", "Split on entering the Wrecked Ship Entrance from the lower door of West Ocean");
    settings.Add("redTowerMiddleEntrance", false, "Red Tower Middle Entrance", "areaTransitions");
    settings.SetToolTip("redTowerMiddleEntrance", "Split on entering Red Tower from Noob Bridge");
    settings.Add("redTowerBottomEntrance", false, "Red Tower Bottom Entrance", "areaTransitions");
    settings.SetToolTip("redTowerBottomEntrance", "Split on entering Red Tower from Skree Boost room");
    settings.Add("kraidsLair", false, "Kraid's Lair", "areaTransitions");
    settings.SetToolTip("kraidsLair", "Split on entering Kraid's Lair");
    settings.Add("risingTideEntrance", false, "Rising Tide Entrance", "areaTransitions");
    settings.SetToolTip("risingTideEntrance", "Split on entering Rising Tide from Cathedral");
    settings.Add("atticExit", false, "Attic Exit", "areaTransitions");
    settings.SetToolTip("atticExit", "Split on exiting Attic");
    settings.Add("tubeBroken", false, "Tube Broken", "areaTransitions");
    settings.SetToolTip("tubeBroken", "Split on blowing up the tube to enter Maridia");
    settings.Add("cacExit", false, "Cacatack Alley Exit", "areaTransitions");
    settings.SetToolTip("cacExit", "Split on exiting West Cacattack Alley");
    settings.Add("toilet", false, "Toilet Bowl", "areaTransitions");
    settings.SetToolTip("toilet", "Split on entering Toilet Bowl from either direction");
    settings.Add("kronicBoost", false, "Kronic Boost Room", "areaTransitions");
    settings.SetToolTip("kronicBoost", "Split on entering Kronic Boost room");
    settings.Add("lowerNorfairEntrance", false, "Lower Norfair Entrance", "areaTransitions");
    settings.SetToolTip("lowerNorfairEntrance", "Split on the elevator down to Lower Norfair");
    settings.Add("writg", false, "Worst Room in the Game", "areaTransitions");
    settings.SetToolTip("writg", "Split on entering Worst Room in the Game");
    settings.Add("redKiShaft", false, "Red Kihunter Shaft", "areaTransitions");
    settings.SetToolTip("redKiShaft", "Split on entering Red Kihunter Shaft from either Amphitheatre or Wastelands (NOTE: will split twice)");
    settings.Add("metalPirates", false, "Metal Pirates Room", "areaTransitions");
    settings.SetToolTip("metalPirates", "Split on entering Metal Pirates Room from Wasteland");
    settings.Add("lowerNorfairSpringMaze", false, "Lower Norfair Springball Maze Room", "areaTransitions");
    settings.SetToolTip("lowerNorfairSpringMaze", "Split on entering Lower Norfair Springball Maze Room");
    settings.Add("lowerNorfairExit", false, "Lower Norfair Exit", "areaTransitions");
    settings.SetToolTip("lowerNorfairExit", "Split on moving from the Three Musketeers' Room to the Single Chamber");
    settings.Add("goldenFour", true, "Golden Four", "areaTransitions");
    settings.SetToolTip("goldenFour", "Split on entering the Statues Room with all four major bosses defeated");
    settings.Add("tourianEntrance", false, "Tourian Entrance", "areaTransitions");
    settings.SetToolTip("tourianEntrance", "Split on the elevator down to Tourian");
    settings.Add("metroids", false, "Tourian Metroid Rooms", "areaTransitions");
    settings.SetToolTip("metroids", "Split on exiting each of the Metroid rooms in Tourian");
    settings.Add("babyMetroidRoom", false, "Baby Metroid Room", "areaTransitions");
    settings.SetToolTip("babyMetroidRoom", "Split on moving from the Dust Torizo Room to the Big Boy Room");
    settings.Add("escapeClimb", false, "Tourian Exit", "areaTransitions");
    settings.SetToolTip("escapeClimb", "Split on moving from Tourian Escape Room 4 to The Climb");

    settings.Add("miniBosses", false, "Minibosses");
    settings.SetToolTip("miniBosses", "Split on defeating minibosses");
    settings.Add("ceresRidley", false, "Ceres Ridley", "miniBosses");
    settings.SetToolTip("ceresRidley", "Split on starting the Ceres Escape");
    settings.Add("bombTorizo", false, "Bomb Torizo", "miniBosses");
    settings.SetToolTip("bombTorizo", "Split on Bomb Torizo's drops appearing");
    settings.Add("sporeSpawn", false, "Spore Spawn", "miniBosses");
    settings.SetToolTip("sporeSpawn", "Split on the last hit to Spore Spawn");
    settings.Add("crocomire", false, "Crocomire", "miniBosses");
    settings.SetToolTip("crocomire", "Split on Crocomire's drops appearing");
    settings.Add("botwoon", false, "Botwoon", "miniBosses");
    settings.SetToolTip("botwoon", "Split on Botwoon's vertical column being fully destroyed");
    settings.Add("goldenTorizo", false, "Golden Torizo", "miniBosses");
    settings.SetToolTip("goldenTorizo", "Split on Golden Torizo's drops appearing");

    settings.Add("bosses", true, "Bosses");
    settings.SetToolTip("bosses", "Split on defeating major bosses");
    settings.Add("kraid", false, "Kraid", "bosses");
    settings.SetToolTip("kraid", "Split shortly after Kraid's drops appear");
    settings.Add("phantoon", false, "Phantoon", "bosses");
    settings.SetToolTip("phantoon", "Split on Phantoon's drops appearing");
    settings.Add("draygon", false, "Draygon", "bosses");
    settings.SetToolTip("draygon", "Split on Draygon's drops appearing");
    settings.Add("ridley", true, "Ridley", "bosses");
    settings.SetToolTip("ridley", "Split on Ridley's drops appearing");
    settings.Add("mb1", false, "Mother Brain 1", "bosses");
    settings.SetToolTip("mb1", "Split on Mother Brain's head hitting the ground at the end of the first phase");
    settings.Add("mb2", true, "Mother Brain 2", "bosses");
    settings.SetToolTip("mb2", "Split on the Baby Metroid detaching from Mother Brain's head");
    settings.Add("mb3", false, "Mother Brain 3", "bosses");
    settings.SetToolTip("mb3", "Split on the start of the Zebes Escape");

    settings.Add("rtaFinish", true, "RTA Finish");
    settings.SetToolTip("rtaFinish", "Split on facing forward at the end of Zebes Escape");
    settings.Add("igtFinish", false, "IGT Finish");
    settings.SetToolTip("igtFinish", "Split on In-Game Time finalizing, when the end cutscene starts");
    settings.Add("sporeSpawnRTAFinish", false, "Spore Spawn RTA Finish");
    settings.SetToolTip("sporeSpawnRTAFinish", "Split on the end of a Spore Spawn RTA run, when the text box clears after collecting the Super Missiles");
    settings.Add("hundredMissileRTAFinish", false, "100 Missile RTA Finish");
    settings.SetToolTip("hundredMissileRTAFinish", "Split on the end of a 100 Missile RTA run, when the text box clears after collecting the hundredth missile");

    // RoomIDs compiled here:
    // https://wiki.supermetroid.run/List_of_rooms_by_SMILE_ID
    vars.roomIDEnum = new Dictionary<string, int> {
        { "landingSite",                    0x91F8 },
        { "crateriaPowerBombRoom",          0x93AA },
        { "westOcean",                      0x93FE },
        { "elevatorToMaridia",              0x94CC },
        { "crateriaMoat",                   0x95FF },
        { "elevatorToCaterpillar",          0x962A },
        { "gauntletETankRoom",              0x965B },
        { "climb",                          0x96BA },
        { "pitRoom",                        0x975C },
        { "elevatorToMorphBall",            0x97B5 },
        { "bombTorizo",                     0x9804 },
        { "terminator",                     0x990D },
        { "elevatorToGreenBrinstar",        0x9938 },
        { "greenPirateShaft",               0x99BD },
        { "crateriaSupersRoom",             0x99F9 },
        { "theFinalMissile",                0x9A90 },
        { "greenBrinstarMainShaft",         0x9AD9 },
        { "sporeSpawnSuper",                0x9B5B },
        { "earlySupers",                    0x9BC8 },
        { "brinstarReserveRoom",            0x9C07 },
        { "bigPink",                        0x9D19 },
        { "sporeSpawnKeyhunter",            0x9D9C },
        { "sporeSpawn",                     0x9DC7 },
        { "pinkBrinstarPowerBombRoom",      0x9E11 },
        { "greenHills",                     0x9E52 },
        { "noobBridge",                     0x9FBA },
        { "morphBall",                      0x9E9F },
        { "blueBrinstarETankRoom",          0x9F64 },
        { "etacoonETankRoom",               0xA011 },
        { "etacoonSuperRoom",               0xA051 },
        { "waterway",                       0xA0D2 },
        { "alphaMissileRoom",               0xA107 },
        { "hopperETankRoom",                0xA15B },
        { "billyMays",                      0xA1D8 },
        { "redTower",                       0xA253 },
        { "xRay",                           0xA2CE },
        { "caterpillar",                    0xA322 },
        { "betaPowerBombRoom",              0xA37C },
        { "alphaPowerBombsRoom",            0xA3AE },
        { "bat",                            0xA3DD },
        { "spazer",                         0xA447 },
        { "warehouseETankRoom",             0xA4B1 },
        { "warehouseZeela",                 0xA471 },
        { "warehouseKiHunters",             0xA4DA },
        { "kraidEyeDoor",                   0xA56B },
        { "kraid",                          0xA59F },
        { "statuesHallway",                 0xA5ED },
        { "statues",                        0xA66A },
        { "warehouseEntrance",              0xA6A1 },
        { "varia",                          0xA6E2 },
        { "cathedral",                      0xA788 },
        { "businessCenter",                 0xA7DE },
        { "iceBeam",                        0xA890 },
        { "crumbleShaft",                   0xA8F8 },
        { "crocomireSpeedway",              0xA923 },
        { "crocomire",                      0xA98D },
        { "hiJump",                         0xA9E5 },
        { "crocomireEscape",                0xAA0E },
        { "hiJumpShaft",                    0xAA41 },
        { "postCrocomirePowerBombRoom",     0xAADE },
        { "cosineRoom",                     0xAB3B },
        { "preGrapple",                     0xAB8F },
        { "grapple",                        0xAC2B },
        { "norfairReserveRoom",             0xAC5A },
        { "greenBubblesRoom",               0xAC83 },
        { "bubbleMountain",                 0xACB3 },
        { "speedBoostHall",                 0xACF0 },
        { "speedBooster",                   0xAD1B },
        { "singleChamber",                  0xAD5E }, // Exit room from Lower Norfair, also on the path to Wave
        { "doubleChamber",                  0xADAD },
        { "waveBeam",                       0xADDE },
        { "volcano",                        0xAE32 },
        { "kronicBoost",                    0xAE74 },
        { "magdolliteTunnel",               0xAEB4 },
        { "lowerNorfairElevator",           0xAF3F },
        { "risingTide",                     0xAFA3 },
        { "spikyAcidSnakes",                0xAFFB },
        { "acidStatue",                     0xB1E5 },
        { "mainHall",                       0xB236 }, // First room in Lower Norfair
        { "goldenTorizo",                   0xB283 },
        { "ridley",                         0xB32E },
        { "lowerNorfairFarming",            0xB37A },
        { "mickeyMouse",                    0xB40A },
        { "pillars",                        0xB457 },
        { "writg",                          0xB4AD },
        { "amphitheatre",                   0xB4E5 },
        { "lowerNorfairSpringMaze",         0xB510 },
        { "lowerNorfairEscapePowerBombRoom",0xB55A },
        { "redKiShaft",                     0xB585 },
        { "wasteland",                      0xB5D5 },
        { "metalPirates",                   0xB62B },
        { "threeMusketeers",                0xB656 },
        { "ridleyETankRoom",                0xB698 },
        { "screwAttack",                    0xB6C1 },
        { "lowerNorfairFireflea",           0xB6EE },
        { "bowling",                        0xC98E },
        { "wreckedShipEntrance",            0xCA08 },
        { "attic",                          0xCA52 },
        { "atticWorkerRobotRoom",           0xCAAE },
        { "wreckedShipMainShaft",           0xCAF6 },
        { "wreckedShipETankRoom",           0xCC27 },
        { "basement",                       0xCC6F }, // Basement of Wrecked Ship
        { "phantoon",                       0xCD13 },
        { "wreckedShipLeftSuperRoom",       0xCDA8 },
        { "wreckedShipRightSuperRoom",      0xCDF1 },
        { "gravity",                        0xCE40 },
        { "glassTunnel",                    0xCEFB },
        { "mainStreet",                     0xCFC9 },
        { "mamaTurtle",                     0xD055 },
        { "wateringHole",                   0xD13B },
        { "beach",                          0xD1DD },
        { "plasmaBeam",                     0xD2AA },
        { "maridiaElevator",                0xD30B },
        { "plasmaSpark",                    0xD340 },
        { "toiletBowl",                     0xD408 },
        { "oasis",                          0xD48E },
        { "leftSandPit",                    0xD4EF },
        { "rightSandPit",                   0xD51E },
        { "aqueduct",                       0xD5A7 },
        { "butterflyRoom",                  0xD5EC },
        { "botwoonHallway",                 0xD617 },
        { "springBall",                     0xD6D0 },
        { "precious",                       0xD78F },
        { "botwoonETankRoom",               0xD7E4 },
        { "botwoon",                        0xD95E },
        { "spaceJump",                      0xD9AA },
        { "westCactusAlley",                0xD9FE },
        { "draygon",                        0xDA60 },
        { "tourianElevator",                0xDAAE },
        { "metroidOne",                     0xDAE1 },
        { "metroidTwo",                     0xDB31 },
        { "metroidThree",                   0xDB7D },
        { "metroidFour",                    0xDBCD },
        { "dustTorizo",                     0xDC65 },
        { "tourianHopper",                  0xDC19 },
        { "tourianEyeDoor",                 0xDDC4 },
        { "bigBoy",                         0xDCB1 },
        { "motherBrain",                    0xDD58 },
        { "rinkaShaft",                     0xDDF3 },
        { "tourianEscape4",                 0xDEDE },
        { "ceresElevator",                  0xDF45 },
        { "flatRoom",                       0xE06B }, // Placeholder name for the flat room in Ceres Station
        { "ceresRidley",                    0xE0B5 }
    };

    vars.mapInUseEnum = new Dictionary<string, int>{
        { "crateria",   0x0 },
        { "brinstar",   0x1 },
        { "norfair",    0x2 },
        { "wreckedShip",0x3 },
        { "maridia",    0x4 },
        { "tourian",    0x5 },
        { "ceres",      0x6 }
    };

    vars.gameStateEnum = new Dictionary<string, int> {
        { "normalGameplay",         0x8 },
        { "doorTransition",         0xB },
        { "startOfCeresCutscene",   0x20 },
        { "preEndCutscene",         0x26 }, // briefly at this value during the black screen transition after the ship fades out
        { "endCutscene",            0x27 }
    };

    vars.unlockFlagEnum = new Dictionary<string, int>{
        // First item byte
        { "variaSuit",      0x1 },
        { "springBall",     0x2 },
        { "morphBall",      0x4 },
        { "screwAttack",    0x8 },
        { "gravSuit",       0x20},
        // Second item byte
        { "hiJump",         0x1 },
        { "spaceJump",      0x2 },
        { "bomb",           0x10},
        { "speedBooster",   0x20},
        { "grapple",        0x40},
        { "xray",           0x80},
        // Beams
        { "wave",           0x1 },
        { "ice",            0x2 },
        { "spazer",         0x4 },
        { "plasma",         0x8 },
        // Charge
        { "chargeBeam",     0x10}
    };

    vars.motherBrainMaxHPEnum = new Dictionary<string, int>{
        { "phase1", 0xBB8 },    // 3000
        { "phase2", 0x4650 },   // 18000
        { "phase3", 0x8CA0 }    // 36000
    };

    vars.eventFlagEnum = new Dictionary<string, int>{
        { "zebesAblaze",    0x40 },
        { "tubeBroken",     0x8 }
    };

    vars.bossFlagEnum = new Dictionary<string, int>{
        // Crateria
        { "bombTorizo",     0x4 },
        // Brinstar
        { "sporeSpawn",     0x2 },
        { "kraid",          0x1 },
        // Norfair
        { "ridley",         0x1 },
        { "crocomire",      0x2 },
        { "goldenTorizo",   0x4 },
        // Wrecked Ship
        { "phantoon",       0x1 },
        // Maridia
        { "draygon",        0x1 },
        { "botwoon",        0x2 },
        // Tourian
        { "motherBrain",    0x2 },
        // Ceres
        { "ceresRidley",    0x1 }
    };

    vars.pickedUpSporeSpawnSuper = false;
    vars.pickedUpHundredthMissile = false;
    vars.frameRate = 60.0;

    Action<string> DebugOutput = (text) => {
        print("[Super Metroid Autosplitter] "+text);
    };
    vars.DebugOutput = DebugOutput;
}

init
{
    IntPtr memoryOffset = IntPtr.Zero;

    if (memory.ProcessName.ToLower().Contains("snes9x")) {
        // TODO: These should probably be module-relative offsets too. Then
        // some of this codepath can be unified with the RA stuff below.
        var versions = new Dictionary<int, long>{
            { 10330112, 0x789414 },   // snes9x 1.52-rr
            { 7729152, 0x890EE4 },    // snes9x 1.54-rr
            { 5914624, 0x6EFBA4 },    // snes9x 1.53
            { 6909952, 0x140405EC8 }, // snes9x 1.53 (x64)
            { 6447104, 0x7410D4 },    // snes9x 1.54/1.54.1
            { 7946240, 0x1404DAF18 }, // snes9x 1.54/1.54.1 (x64)
            { 6602752, 0x762874 },    // snes9x 1.55
            { 8355840, 0x1405BFDB8 }, // snes9x 1.55 (x64)
            { 6856704, 0x78528C },    // snes9x 1.56/1.56.2
            { 9003008, 0x1405D8C68 }, // snes9x 1.56 (x64)
            { 6848512, 0x7811B4 },    // snes9x 1.56.1
            { 8945664, 0x1405C80A8 }, // snes9x 1.56.1 (x64)
            { 9015296, 0x1405D9298 }, // snes9x 1.56.2 (x64)
            { 6991872, 0x7A6EE4 },    // snes9x 1.57
            { 9048064, 0x1405ACC58 }, // snes9x 1.57 (x64)
            { 7000064, 0x7A7EE4 },    // snes9x 1.58
            { 9060352, 0x1405AE848 }, // snes9x 1.58 (x64)
            { 8953856, 0x975A54 },    // snes9x 1.59.2
            { 12537856, 0x1408D86F8 },// snes9x 1.59.2 (x64)
            { 9646080, 0x97EE04 },    // Snes9x-rr 1.60
            { 13565952, 0x140925118 },// Snes9x-rr 1.60 (x64)
            { 9027584, 0x94DB54 },    // snes9x 1.60
            { 12836864, 0x1408D8BE8 } // snes9x 1.60 (x64)
        };

        long pointerAddr;
        if (versions.TryGetValue(modules.First().ModuleMemorySize, out pointerAddr)) {
            memoryOffset = memory.ReadPointer((IntPtr)pointerAddr);
        }
    } else if (memory.ProcessName.ToLower().Contains("higan") || memory.ProcessName.ToLower().Contains("bsnes") || memory.ProcessName.ToLower().Contains("emuhawk") || memory.ProcessName.ToLower().Contains("lsnes-bsnes")) {
        var versions = new Dictionary<int, long>{
            { 12509184, 0x915304 },      // higan v102
            { 13062144, 0x937324 },      // higan v103
            { 15859712, 0x952144 },      // higan v104
            { 16756736, 0x94F144 },      // higan v105tr1
            { 16019456, 0x94D144 },      // higan v106
            { 15360000, 0x8AB144 },      // higan v106.112
            { 22388736, 0xB0ECC8 },      // higan v107
            { 23142400, 0xBC7CC8 },      // higan v108
            { 23166976, 0xBCECC8 },      // higan v109
            { 23224320, 0xBDBCC8 },      // higan v110
            { 10096640, 0x72BECC },      // bsnes v107
            { 10338304, 0x762F2C },      // bsnes v107.1
            { 47230976, 0x765F2C },      // bsnes v107.2/107.3
            { 142282752, 0xA65464 },     // bsnes v108
            { 131354624, 0xA6ED5C },     // bsnes v109
            { 131543040, 0xA9BD5C },     // bsnes v110
            { 51924992, 0xA9DD5C },      // bsnes v111
            { 52056064, 0xAAED7C },      // bsnes v112
            // Unfortunately v113/114 cannot be supported with this style
            // of check because their size matches v115, with a different offset
            //{ 52477952, 0xB15D7C },      // bsnes v113/114
            { 52477952, 0xB16D7C },      // bsnes v115
            { 7061504,  0x36F11500240 }, // BizHawk 2.3
            { 7249920,  0x36F11500240 }, // BizHawk 2.3.1
            { 6938624,  0x36F11500240 }, // BizHawk 2.3.2
            { 35414016, 0x023A1BF0 },    // lsnes rr2-B23
        };

        long wramAddr;
        if (versions.TryGetValue(modules.First().ModuleMemorySize, out wramAddr)) {
            memoryOffset = (IntPtr)wramAddr;
        }
    } else if (memory.ProcessName.ToLower().Contains("retroarch")) {
        // RetroArch stores a pointer to the emulated WRAM inside itself (it
        // can get this pointer via the Core API). This happily lets this work
        // on any variant of Snes9x cores, depending only on the RA version.

        var retroarchVersions = new Dictionary<int, int>{
            { 18649088, 0x608EF0 }, // Retroarch 1.7.5 (x64)
        };
        IntPtr wramPointer = IntPtr.Zero;
        int ptrOffset;
        if (retroarchVersions.TryGetValue(modules.First().ModuleMemorySize, out ptrOffset)) {
            wramPointer = memory.ReadPointer(modules.First().BaseAddress + ptrOffset);
        }

        if (wramPointer != IntPtr.Zero) {
            memoryOffset = wramPointer;
        } else {
            // Unfortunately, Higan doesn't support that API. So if the address
            // is missing, try to grab the memory from the higan core directly.

            var higanModule = modules.FirstOrDefault(m => m.ModuleName.ToLower() == "higan_sfc_libretro.dll");
            if (higanModule != null) {
                var versions = new Dictionary<int, int>{
                    { 4980736, 0x1F3AC4 }, // higan 106 (x64)
                };
                int wramOffset;
                if (versions.TryGetValue(higanModule.ModuleMemorySize, out wramOffset)) {
                    memoryOffset = higanModule.BaseAddress + wramOffset;
                }
            }
        }
    }

    if (memoryOffset == IntPtr.Zero) {
        vars.DebugOutput("Unsupported emulator version");
        var interestingModules = modules.Where(m =>
            m.ModuleName.ToLower().EndsWith(".exe") ||
            m.ModuleName.ToLower().EndsWith("_libretro.dll"));
        foreach (var module in interestingModules) {
            vars.DebugOutput("Module '" + module.ModuleName + "' sized " + module.ModuleMemorySize.ToString());
        }
        vars.watchers = new MemoryWatcherList{};
        // Throwing prevents initialization from completing. LiveSplit will
        // retry it until it eventually works. (Which lets you load a core in
        // RA for example.)
        throw new InvalidOperationException("Unsupported emulator version");
    }

    vars.DebugOutput("Found WRAM address: 0x" + memoryOffset.ToString("X8"));
    vars.watchers = new MemoryWatcherList
    {
        new MemoryWatcher<ushort>(memoryOffset + 0x079B) { Name = "roomID" },
        new MemoryWatcher<byte>(memoryOffset + 0x079F) { Name = "mapInUse" },
        new MemoryWatcher<byte>(memoryOffset + 0x0998) { Name = "gameState" },
        new MemoryWatcher<byte>(memoryOffset + 0x09A4) { Name = "unlockedEquips2" },
        new MemoryWatcher<byte>(memoryOffset + 0x09A5) { Name = "unlockedEquips" },
        new MemoryWatcher<byte>(memoryOffset + 0x09A8) { Name = "unlockedBeams" },
        new MemoryWatcher<byte>(memoryOffset + 0x09A9) { Name = "unlockedCharge" },
        new MemoryWatcher<ushort>(memoryOffset + 0x09C4) { Name = "maxEnergy" },
        new MemoryWatcher<byte>(memoryOffset + 0x09C8) { Name = "maxMissiles" },
        new MemoryWatcher<byte>(memoryOffset + 0x09CC) { Name = "maxSupers" },
        new MemoryWatcher<byte>(memoryOffset + 0x09D0) { Name = "maxPowerBombs" },
        new MemoryWatcher<ushort>(memoryOffset + 0x09D4) { Name = "maxReserve" },
        new MemoryWatcher<byte>(memoryOffset + 0x09DA) { Name = "igtFrames" },
        new MemoryWatcher<byte>(memoryOffset + 0x09DC) { Name = "igtSeconds" },
        new MemoryWatcher<byte>(memoryOffset + 0x09DE) { Name = "igtMinutes" },
        new MemoryWatcher<byte>(memoryOffset + 0x09E0) { Name = "igtHours" },
        new MemoryWatcher<byte>(memoryOffset + 0x0A28) { Name = "playerState" },
        new MemoryWatcher<ushort>(memoryOffset + 0x0F8C) { Name = "enemyHP" },
        new MemoryWatcher<ushort>(memoryOffset + 0x0FB2) { Name = "shipAI" },
        new MemoryWatcher<ushort>(memoryOffset + 0x0FCC) { Name = "motherBrainHP" },
        new MemoryWatcher<byte>(memoryOffset + 0xD821) { Name = "eventFlags" },
        new MemoryWatcher<byte>(memoryOffset + 0xD828) { Name = "crateriaBosses" },
        new MemoryWatcher<byte>(memoryOffset + 0xD829) { Name = "brinstarBosses" },
        new MemoryWatcher<byte>(memoryOffset + 0xD82A) { Name = "norfairBosses" },
        new MemoryWatcher<byte>(memoryOffset + 0xD82B) { Name = "wreckedShipBosses" },
        new MemoryWatcher<byte>(memoryOffset + 0xD82C) { Name = "maridiaBosses" },
        new MemoryWatcher<byte>(memoryOffset + 0xD82D) { Name = "tourianBosses" },
        new MemoryWatcher<byte>(memoryOffset + 0xD82E) { Name = "ceresBosses" },
        new MemoryWatcher<byte>(memoryOffset + 0xD870) { Name = "crateriaItems" },
        new MemoryWatcher<byte>(memoryOffset + 0xD871) { Name = "brinteriaItems" },
        new MemoryWatcher<byte>(memoryOffset + 0xD872) { Name = "brinstarItems2" },
        new MemoryWatcher<byte>(memoryOffset + 0xD873) { Name = "brinstarItems3" },
        new MemoryWatcher<byte>(memoryOffset + 0xD874) { Name = "brinstarItems4" },
        new MemoryWatcher<byte>(memoryOffset + 0xD875) { Name = "brinstarItems5" },
        new MemoryWatcher<byte>(memoryOffset + 0xD876) { Name = "norfairItems1" },
        new MemoryWatcher<byte>(memoryOffset + 0xD877) { Name = "norfairItems2" },
        new MemoryWatcher<byte>(memoryOffset + 0xD878) { Name = "norfairItems3" },
        new MemoryWatcher<byte>(memoryOffset + 0xD879) { Name = "norfairItems4" },
        new MemoryWatcher<byte>(memoryOffset + 0xD87A) { Name = "norfairItems5" },
        new MemoryWatcher<byte>(memoryOffset + 0xD880) { Name = "wreckedShipItems" },
        new MemoryWatcher<byte>(memoryOffset + 0xD881) { Name = "maridiaItems1" },
        new MemoryWatcher<byte>(memoryOffset + 0xD882) { Name = "maridiaItems2" },
        new MemoryWatcher<byte>(memoryOffset + 0xD883) { Name = "maridiaItems3" },
    };
}

update
{
    vars.watchers.UpdateAll(game);
}

start
{
    var normalStart   = vars.watchers["gameState"].Old == 2    && vars.watchers["gameState"].Current == 0x1F;
    // Allow for a cutscene start, even though it's not normally used for speedrunning
    var cutsceneEnded = vars.watchers["gameState"].Old == 0x1E && vars.watchers["gameState"].Current == 0x1F;
    // Some categories start from Zebes, such as Spore Spawn RTA
    var zebesStart    = vars.watchers["gameState"].Old == 5    && vars.watchers["gameState"].Current == 6;
    if (normalStart || cutsceneEnded || zebesStart) {
        vars.DebugOutput("Timer started");
    }
    return normalStart || cutsceneEnded || zebesStart;
}

reset
{
    return vars.watchers["roomID"].Old != 0 && vars.watchers["roomID"].Current == 0;
}

split
{
    // Ammo pickup section
    var firstMissile = settings["firstMissile"] && vars.watchers["maxMissiles"].Old == 0 && vars.watchers["maxMissiles"].Current == 5;
    var allMissiles = settings["allMissiles"] && (vars.watchers["maxMissiles"].Old + 5) == (vars.watchers["maxMissiles"].Current);
    var oceanBottomMissiles = settings["oceanBottomMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["westOcean"] && (vars.watchers["crateriaItems"].Old + 2) == (vars.watchers["crateriaItems"].Current);
    var oceanTopMissiles = settings["oceanTopMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["westOcean"] && (vars.watchers["crateriaItems"].Old + 4) == (vars.watchers["crateriaItems"].Current);
    var oceanMiddleMissiles = settings["oceanMiddleMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["westOcean"] && (vars.watchers["crateriaItems"].Old + 8) == (vars.watchers["crateriaItems"].Current);
    var moatMissiles = settings["moatMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["crateriaMoat"] && (vars.watchers["crateriaItems"].Old + 16) == (vars.watchers["crateriaItems"].Current);
    var oldTourianMissiles = settings["oldTourianMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["pitRoom"] && (vars.watchers["crateriaItems"].Old + 64) == (vars.watchers["crateriaItems"].Current);
    var gauntletRightMissiles = settings["gauntletRightMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["greenPirateShaft"] && (vars.watchers["brinteriaItems"].Old + 2) == (vars.watchers["brinteriaItems"].Current);
    var gauntletLeftMissiles = settings["gauntletLeftMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["greenPirateShaft"] && (vars.watchers["brinteriaItems"].Old + 4) == (vars.watchers["brinteriaItems"].Current);
    var dentalPlan = settings["dentalPlan"] && vars.watchers["roomID"].Current == vars.roomIDEnum["theFinalMissile"] && (vars.watchers["brinteriaItems"].Old + 16) == (vars.watchers["brinteriaItems"].Current);
    var earlySuperBridgeMissiles = settings["earlySuperBridgeMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["earlySupers"] && (vars.watchers["brinteriaItems"].Old + 128) == (vars.watchers["brinteriaItems"].Current);
    var greenBrinstarReserveMissiles = settings["greenBrinstarReserveMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["brinstarReserveRoom"] && (vars.watchers["brinstarItems2"].Old + 8) == (vars.watchers["brinstarItems2"].Current);
    var greenBrinstarExtraReserveMissiles = settings["greenBrinstarExtraReserveMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["brinstarReserveRoom"] && (vars.watchers["brinstarItems2"].Old + 4) == (vars.watchers["brinstarItems2"].Current);
    var bigPinkTopMissiles = settings["bigPinkTopMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["bigPink"] && (vars.watchers["brinstarItems2"].Old + 32) == (vars.watchers["brinstarItems2"].Current);
    var chargeMissiles = settings["chargeMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["bigPink"] && (vars.watchers["brinstarItems2"].Old + 64) == (vars.watchers["brinstarItems2"].Current);
    var greenHillsMissiles = settings["greenHillsMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["greenHills"] && (vars.watchers["brinstarItems3"].Old + 2) == (vars.watchers["brinstarItems3"].Current);
    var blueBrinstarETankMissiles = settings["blueBrinstarETankMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["blueBrinstarETankRoom"] && (vars.watchers["brinstarItems3"].Old + 16) == (vars.watchers["brinstarItems3"].Current);
    var alphaMissiles = settings["alphaMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["alphaMissileRoom"] && (vars.watchers["brinstarItems4"].Old + 4) == (vars.watchers["brinstarItems4"].Current);
    var billyMaysMissiles = settings["billyMaysMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["billyMays"] && (vars.watchers["brinstarItems4"].Old + 16) == (vars.watchers["brinstarItems4"].Current);
    var butWaitTheresMoreMissiles = settings["butWaitTheresMoreMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["billyMays"] && (vars.watchers["brinstarItems4"].Old + 32) == (vars.watchers["brinstarItems4"].Current);
    var redBrinstarMissiles = settings["redBrinstarMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["alphaPowerBombsRoom"] && (vars.watchers["brinstarItems5"].Old + 2) == (vars.watchers["brinstarItems5"].Current);
    var warehouseMissiles = settings["warehouseMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["warehouseKiHunters"] && (vars.watchers["brinstarItems5"].Old + 16) == (vars.watchers["brinstarItems5"].Current);
    var cathedralMissiles = settings["cathedralMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["cathedral"] && (vars.watchers["norfairItems1"].Old + 2) == (vars.watchers["norfairItems1"].Current);
    var crumbleShaftMissiles = settings["crumbleShaftMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["crumbleShaft"] && (vars.watchers["norfairItems1"].Old + 8) == (vars.watchers["norfairItems1"].Current);
    var crocomireEscapeMissiles = settings["crocomireEscapeMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["crocomireEscape"] && (vars.watchers["norfairItems1"].Old + 64) == (vars.watchers["norfairItems1"].Current);
    var hiJumpMissiles = settings["hiJumpMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["hiJumpShaft"] && (vars.watchers["norfairItems1"].Old + 128) == (vars.watchers["norfairItems1"].Current);
    var postCrocomireMissiles = settings["postCrocomireMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["cosineRoom"] && (vars.watchers["norfairItems2"].Old + 4) == (vars.watchers["norfairItems2"].Current);
    var grappleMissiles = settings["grappleMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["preGrapple"] && (vars.watchers["norfairItems2"].Old + 8) == (vars.watchers["norfairItems2"].Current);
    var norfairReserveMissiles = settings["norfairReserveMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["norfairReserveRoom"] && (vars.watchers["norfairItems2"].Old + 64) == (vars.watchers["norfairItems2"].Current);
    var greenBubblesMissiles = settings["greenBubblesMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["greenBubblesRoom"] && (vars.watchers["norfairItems2"].Old + 128) == (vars.watchers["norfairItems2"].Current);
    var bubbleMountainMissiles = settings["bubbleMountainMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["bubbleMountain"] && (vars.watchers["norfairItems3"].Old + 1) == (vars.watchers["norfairItems3"].Current);
    var speedBoostMissiles = settings["speedBoostMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["speedBoostHall"] && (vars.watchers["norfairItems3"].Old + 2) == (vars.watchers["norfairItems3"].Current);
    var waveMissiles = settings["waveMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["doubleChamber"] && (vars.watchers["norfairItems3"].Old + 8) == (vars.watchers["norfairItems3"].Current);
    var goldTorizoMissiles = settings["goldTorizoMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["goldenTorizo"] && (vars.watchers["norfairItems3"].Old + 64) == (vars.watchers["norfairItems3"].Current);
    var mickeyMouseMissiles = settings["mickeyMouseMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["mickeyMouse"] && (vars.watchers["norfairItems4"].Old + 2) == (vars.watchers["norfairItems4"].Current);
    var lowerNorfairSpringMazeMissiles = settings["lowerNorfairSpringMazeMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["lowerNorfairSpringMaze"] && (vars.watchers["norfairItems4"].Old + 4) == (vars.watchers["norfairItems4"].Current);
    var threeMusketeersMissiles = settings["threeMusketeersMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["threeMusketeers"] && (vars.watchers["norfairItems4"].Old + 32) == (vars.watchers["norfairItems4"].Current);
    var wreckedShipMainShaftMissiles = settings["wreckedShipMainShaftMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["wreckedShipMainShaft"] && (vars.watchers["wreckedShipItems"].Old + 1) == (vars.watchers["wreckedShipItems"].Current);
    var bowlingMissiles = settings["bowlingMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["bowling"] && (vars.watchers["wreckedShipItems"].Old + 4) == (vars.watchers["wreckedShipItems"].Current);
    var atticMissiles = settings["atticMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["atticWorkerRobotRoom"] && (vars.watchers["wreckedShipItems"].Old + 8) == (vars.watchers["wreckedShipItems"].Current);
    var mainStreetMissiles = settings["mainStreetMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["mainStreet"] && (vars.watchers["maridiaItems1"].Old + 1) == (vars.watchers["maridiaItems1"].Current);
    var mamaTurtleMissiles = settings["mamaTurtleMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["mamaTurtle"] && (vars.watchers["maridiaItems1"].Old + 8) == (vars.watchers["maridiaItems1"].Current);
    var wateringHoleMissiles = settings["wateringHoleMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["wateringHole"] && (vars.watchers["maridiaItems1"].Old + 32) == (vars.watchers["maridiaItems1"].Current);
    var beachMissiles = settings["beachMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["beach"] && (vars.watchers["maridiaItems1"].Old + 64) == (vars.watchers["maridiaItems1"].Current);
    var leftSandPitMissiles = settings["leftSandPitMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["leftSandPit"] && (vars.watchers["maridiaItems2"].Old + 1) == (vars.watchers["maridiaItems2"].Current);
    var rightSandPitMissiles = settings["rightSandPitMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["rightSandPit"] && (vars.watchers["maridiaItems2"].Old + 4) == (vars.watchers["maridiaItems2"].Current);
    var aqueductMissiles = settings["aqueductMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["aqueduct"] && (vars.watchers["maridiaItems2"].Old + 16) == (vars.watchers["maridiaItems2"].Current);
    var preDraygonMissiles = settings["preDraygonMissiles"] && vars.watchers["roomID"].Current == vars.roomIDEnum["precious"] && (vars.watchers["maridiaItems2"].Old + 128) == (vars.watchers["maridiaItems2"].Current);
    var firstSuper = settings["firstSuper"] && vars.watchers["maxSupers"].Old == 0 && vars.watchers["maxSupers"].Current == 5;
    var allSupers = settings["allSupers"] && (vars.watchers["maxSupers"].Old + 5) == (vars.watchers["maxSupers"].Current);
    var climbSupers = settings["climbSupers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["crateriaSupersRoom"] && (vars.watchers["brinteriaItems"].Old + 8) == (vars.watchers["brinteriaItems"].Current);
    var sporeSpawnSupers = settings["sporeSpawnSupers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["sporeSpawnSuper"] && (vars.watchers["brinteriaItems"].Old + 64) == (vars.watchers["brinteriaItems"].Current);
    var earlySupers = settings["earlySupers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["earlySupers"] && (vars.watchers["brinstarItems2"].Old + 1) == (vars.watchers["brinstarItems2"].Current);
    var etacoonSupers = settings["etacoonSupers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["etacoonSuperRoom"] && (vars.watchers["brinstarItems3"].Old + 128) == (vars.watchers["brinstarItems3"].Current);
    var goldTorizoSupers = settings["goldTorizoSupers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["goldenTorizo"] && (vars.watchers["norfairItems3"].Old + 128) == (vars.watchers["norfairItems3"].Current);
    var wreckedShipLeftSupers = settings["wreckedShipLeftSupers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["wreckedShipLeftSuperRoom"] && (vars.watchers["wreckedShipItems"].Old + 32) == (vars.watchers["wreckedShipItems"].Current);
    var wreckedShipRightSupers = settings["wreckedShipRightSupers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["wreckedShipRightSuperRoom"] && (vars.watchers["wreckedShipItems"].Old + 64) == (vars.watchers["wreckedShipItems"].Current);
    var crabSupers = settings["crabSupers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["mainStreet"] && (vars.watchers["maridiaItems1"].Old + 2) == (vars.watchers["maridiaItems1"].Current);
    var wateringHoleSupers = settings["wateringHoleSupers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["wateringHole"] && (vars.watchers["maridiaItems1"].Old + 16) == (vars.watchers["maridiaItems1"].Current);
    var aqueductSupers = settings["aqueductSupers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["aqueduct"] && (vars.watchers["maridiaItems2"].Old + 32) == (vars.watchers["maridiaItems2"].Current);
    var firstPowerBomb = settings["firstPowerBomb"] && vars.watchers["maxPowerBombs"].Old == 0 && vars.watchers["maxPowerBombs"].Current == 5;
    var allPowerBombs = settings["allPowerBombs"] && (vars.watchers["maxPowerBombs"].Old + 5) == (vars.watchers["maxPowerBombs"].Current);
    var landingSiteBombs = settings["landingSiteBombs"] && vars.watchers["roomID"].Current == vars.roomIDEnum["crateriaPowerBombRoom"] && (vars.watchers["crateriaItems"].Old + 1) == (vars.watchers["crateriaItems"].Current);
    var etacoonBombs = settings["etacoonBombs"] && vars.watchers["roomID"].Current == vars.roomIDEnum["greenBrinstarMainShaft"] && (vars.watchers["brinteriaItems"].Old + 32) == (vars.watchers["brinteriaItems"].Current);
    var pinkBrinstarBombs = settings["pinkBrinstarBombs"] && vars.watchers["roomID"].Current == vars.roomIDEnum["pinkBrinstarPowerBombRoom"] && (vars.watchers["brinstarItems3"].Old + 1) == (vars.watchers["brinstarItems3"].Current);
    var blueBrinstarBombs = settings["blueBrinstarBombs"] && vars.watchers["roomID"].Current == vars.roomIDEnum["morphBall"] && (vars.watchers["brinstarItems3"].Old + 8) == (vars.watchers["brinstarItems3"].Current);
    var alphaBombs = settings["alphaBombs"] && vars.watchers["roomID"].Current == vars.roomIDEnum["alphaPowerBombsRoom"] && (vars.watchers["brinstarItems5"].Old + 1) == (vars.watchers["brinstarItems5"].Current);
    var betaBombs = settings["betaBombs"] && vars.watchers["roomID"].Current == vars.roomIDEnum["betaPowerBombRoom"] && (vars.watchers["brinstarItems4"].Old + 128) == (vars.watchers["brinstarItems4"].Current);
    var crocomireBombs = settings["crocomireBombs"] && vars.watchers["roomID"].Current == vars.roomIDEnum["postCrocomirePowerBombRoom"] && (vars.watchers["norfairItems2"].Old + 2) == (vars.watchers["norfairItems2"].Current);
    var lowerNorfairEscapeBombs = settings["lowerNorfairEscapeBombs"] && vars.watchers["roomID"].Current == vars.roomIDEnum["lowerNorfairEscapePowerBombRoom"] && (vars.watchers["norfairItems4"].Old + 8) == (vars.watchers["norfairItems4"].Current);
    var shameBombs = settings["shameBombs"] && vars.watchers["roomID"].Current == vars.roomIDEnum["wasteland"] && (vars.watchers["norfairItems4"].Old + 16) == (vars.watchers["norfairItems4"].Current);
    var rightSandPitBombs = settings["rightSandPitBombs"] && vars.watchers["roomID"].Current == vars.roomIDEnum["rightSandPit"] && (vars.watchers["maridiaItems2"].Old + 8) == (vars.watchers["maridiaItems2"].Current);
    var pickup = firstMissile || allMissiles || oceanBottomMissiles || oceanTopMissiles ||  oceanMiddleMissiles || moatMissiles || oldTourianMissiles || gauntletRightMissiles || gauntletLeftMissiles || dentalPlan || earlySuperBridgeMissiles || greenBrinstarReserveMissiles || greenBrinstarExtraReserveMissiles || bigPinkTopMissiles || chargeMissiles || greenHillsMissiles || blueBrinstarETankMissiles || alphaMissiles || billyMaysMissiles || butWaitTheresMoreMissiles || redBrinstarMissiles || warehouseMissiles || cathedralMissiles || crumbleShaftMissiles || crocomireEscapeMissiles || hiJumpMissiles || postCrocomireMissiles || grappleMissiles || norfairReserveMissiles || greenBubblesMissiles || bubbleMountainMissiles || speedBoostMissiles || waveMissiles || goldTorizoMissiles || mickeyMouseMissiles || lowerNorfairSpringMazeMissiles || threeMusketeersMissiles || wreckedShipMainShaftMissiles || bowlingMissiles || atticMissiles || mainStreetMissiles || mamaTurtleMissiles || wateringHoleMissiles || beachMissiles || leftSandPitMissiles || rightSandPitMissiles || aqueductMissiles || preDraygonMissiles || firstSuper || allSupers || climbSupers || sporeSpawnSupers || earlySupers || etacoonSupers || goldTorizoSupers || wreckedShipLeftSupers || wreckedShipRightSupers || crabSupers || wateringHoleSupers || aqueductSupers || firstPowerBomb || allPowerBombs || landingSiteBombs || etacoonBombs || pinkBrinstarBombs || blueBrinstarBombs || alphaBombs || betaBombs || crocomireBombs || lowerNorfairEscapeBombs || shameBombs || rightSandPitBombs;

    // Item unlock section
    var varia = settings["variaSuit"] && vars.watchers["roomID"].Current == vars.roomIDEnum["varia"] && (vars.watchers["unlockedEquips2"].Old & vars.unlockFlagEnum["variaSuit"]) == 0 && (vars.watchers["unlockedEquips2"].Current & vars.unlockFlagEnum["variaSuit"]) > 0;
    var springBall = settings["springBall"] && vars.watchers["roomID"].Current == vars.roomIDEnum["springBall"] && (vars.watchers["unlockedEquips2"].Old & vars.unlockFlagEnum["springBall"]) == 0 && (vars.watchers["unlockedEquips2"].Current & vars.unlockFlagEnum["springBall"]) > 0;
    var morphBall = settings["morphBall"] && vars.watchers["roomID"].Current == vars.roomIDEnum["morphBall"] && (vars.watchers["unlockedEquips2"].Old & vars.unlockFlagEnum["morphBall"]) == 0 && (vars.watchers["unlockedEquips2"].Current & vars.unlockFlagEnum["morphBall"]) > 0;
    var screwAttack = settings["screwAttack"] && vars.watchers["roomID"].Current == vars.roomIDEnum["screwAttack"] && (vars.watchers["unlockedEquips2"].Old & vars.unlockFlagEnum["screwAttack"]) == 0 && (vars.watchers["unlockedEquips2"].Current & vars.unlockFlagEnum["screwAttack"]) > 0;
    var gravSuit = settings["gravSuit"] && vars.watchers["roomID"].Current == vars.roomIDEnum["gravity"] && (vars.watchers["unlockedEquips2"].Old & vars.unlockFlagEnum["gravSuit"]) == 0 && (vars.watchers["unlockedEquips2"].Current & vars.unlockFlagEnum["gravSuit"]) > 0;
    var hiJump = settings["hiJump"] && vars.watchers["roomID"].Current == vars.roomIDEnum["hiJump"] && (vars.watchers["unlockedEquips"].Old & vars.unlockFlagEnum["hiJump"]) == 0 && (vars.watchers["unlockedEquips"].Current & vars.unlockFlagEnum["hiJump"]) > 0;
    var spaceJump = settings["spaceJump"] && vars.watchers["roomID"].Current == vars.roomIDEnum["spaceJump"] && (vars.watchers["unlockedEquips"].Old & vars.unlockFlagEnum["spaceJump"]) == 0 && (vars.watchers["unlockedEquips"].Current & vars.unlockFlagEnum["spaceJump"]) > 0;
    var bomb = settings["bomb"] && vars.watchers["roomID"].Current == vars.roomIDEnum["bombTorizo"] && (vars.watchers["unlockedEquips"].Old & vars.unlockFlagEnum["bomb"]) == 0 && (vars.watchers["unlockedEquips"].Current & vars.unlockFlagEnum["bomb"]) > 0;
    var speedBooster = settings["speedBooster"] && vars.watchers["roomID"].Current == vars.roomIDEnum["speedBooster"] && (vars.watchers["unlockedEquips"].Old & vars.unlockFlagEnum["speedBooster"]) == 0 && (vars.watchers["unlockedEquips"].Current & vars.unlockFlagEnum["speedBooster"]) > 0;
    var grapple = settings["grapple"] && vars.watchers["roomID"].Current == vars.roomIDEnum["grapple"] && (vars.watchers["unlockedEquips"].Old & vars.unlockFlagEnum["grapple"]) == 0 && (vars.watchers["unlockedEquips"].Current & vars.unlockFlagEnum["grapple"]) > 0;
    var xray = settings["xray"] && vars.watchers["roomID"].Current == vars.roomIDEnum["xRay"] && (vars.watchers["unlockedEquips"].Old & vars.unlockFlagEnum["xray"]) == 0 && (vars.watchers["unlockedEquips"].Current & vars.unlockFlagEnum["xray"]) > 0;
    var unlock = varia || springBall || morphBall || screwAttack || gravSuit || hiJump || spaceJump || bomb || speedBooster || grapple || xray;

    // Beam unlock section
    var wave = settings["wave"] && vars.watchers["roomID"].Current == vars.roomIDEnum["waveBeam"] && (vars.watchers["unlockedBeams"].Old & vars.unlockFlagEnum["wave"]) == 0 && (vars.watchers["unlockedBeams"].Current & vars.unlockFlagEnum["wave"]) > 0;
    var ice = settings["ice"] && vars.watchers["roomID"].Current == vars.roomIDEnum["iceBeam"] && (vars.watchers["unlockedBeams"].Old & vars.unlockFlagEnum["ice"]) == 0 && (vars.watchers["unlockedBeams"].Current & vars.unlockFlagEnum["ice"]) > 0;
    var spazer = settings["spazer"] && vars.watchers["roomID"].Current == vars.roomIDEnum["spazer"] && (vars.watchers["unlockedBeams"].Old & vars.unlockFlagEnum["spazer"]) == 0 && (vars.watchers["unlockedBeams"].Current & vars.unlockFlagEnum["spazer"]) > 0;
    var plasma = settings["plasma"] && vars.watchers["roomID"].Current == vars.roomIDEnum["plasmaBeam"] && (vars.watchers["unlockedBeams"].Old & vars.unlockFlagEnum["plasma"]) == 0 && (vars.watchers["unlockedBeams"].Current & vars.unlockFlagEnum["plasma"]) > 0;
    var chargeBeam = settings["chargeBeam"] && vars.watchers["roomID"].Current == vars.roomIDEnum["bigPink"] && (vars.watchers["unlockedCharge"].Old & vars.unlockFlagEnum["chargeBeam"]) == 0 && (vars.watchers["unlockedCharge"].Current & vars.unlockFlagEnum["chargeBeam"]) > 0;
    var beam = wave || ice || spazer || plasma || chargeBeam;

    // E-tanks and reserve tanks
    var firstETank = settings["firstETank"] && vars.watchers["maxEnergy"].Old == 99 && vars.watchers["maxEnergy"].Current == 199;
    var allETanks = settings["allETanks"] && (vars.watchers["maxEnergy"].Old + 100) == (vars.watchers["maxEnergy"].Current);
    var gauntletETank = settings["gauntletETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["gauntletETankRoom"] && (vars.watchers["crateriaItems"].Old + 32) == (vars.watchers["crateriaItems"].Current);
    var terminatorETank = settings["terminatorETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["terminator"] && (vars.watchers["brinteriaItems"].Old + 1) == (vars.watchers["brinteriaItems"].Current);
    var ceilingETank = settings["ceilingETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["blueBrinstarETankRoom"] && (vars.watchers["brinstarItems3"].Old + 32) == (vars.watchers["brinstarItems3"].Current);
    var etecoonsETank = settings["etecoonsETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["etacoonETankRoom"] && (vars.watchers["brinstarItems3"].Old + 64) == (vars.watchers["brinstarItems3"].Current);
    var waterwayETank = settings["waterwayETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["waterway"] && (vars.watchers["brinstarItems4"].Old + 2) == (vars.watchers["brinstarItems4"].Current);
    var waveGateETank = settings["waveGateETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["hopperETankRoom"] && (vars.watchers["brinstarItems4"].Old + 8) == (vars.watchers["brinstarItems4"].Current);
    var kraidETank = settings["kraidETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["warehouseETankRoom"] && (vars.watchers["brinstarItems5"].Old + 8) == (vars.watchers["brinstarItems5"].Current);
    var crocomireETank = settings["crocomireETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["crocomire"] && (vars.watchers["norfairItems1"].Old + 16) == (vars.watchers["norfairItems1"].Current);
    var hiJumpETank = settings["hiJumpETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["hiJumpShaft"] && (vars.watchers["norfairItems2"].Old + 1) == (vars.watchers["norfairItems2"].Current);
    var ridleyETank = settings["ridleyETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["ridleyETankRoom"] && (vars.watchers["norfairItems4"].Old + 64) == (vars.watchers["norfairItems4"].Current);
    var firefleaETank = settings["firefleaETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["lowerNorfairFireflea"] && (vars.watchers["norfairItems5"].Old + 1) == (vars.watchers["norfairItems5"].Current);
    var wreckedShipETank = settings["wreckedShipETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["wreckedShipETankRoom"] && (vars.watchers["wreckedShipItems"].Old + 16) == (vars.watchers["wreckedShipItems"].Current);
    var tatoriETank = settings["tatoriETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["mamaTurtle"] && (vars.watchers["maridiaItems1"].Old + 4) == (vars.watchers["maridiaItems1"].Current);
    var botwoonETank = settings["botwoonETank"] && vars.watchers["roomID"].Current == vars.roomIDEnum["botwoonETankRoom"] && (vars.watchers["maridiaItems3"].Old + 1) == (vars.watchers["maridiaItems3"].Current);
    var reserveTanks = settings["reserveTanks"] && (vars.watchers["maxReserve"].Old + 100) == (vars.watchers["maxReserve"].Current);
    var brinstarReserve = settings["brinstarReserve"] && vars.watchers["roomID"].Current == vars.roomIDEnum["brinstarReserveRoom"] && (vars.watchers["brinstarItems2"].Old + 2) == (vars.watchers["brinstarItems2"].Current);
    var norfairReserve = settings["norfairReserve"] && vars.watchers["roomID"].Current == vars.roomIDEnum["norfairReserveRoom"] && (vars.watchers["norfairItems2"].Old + 32) == (vars.watchers["norfairItems2"].Current);
    var wreckedShipReserve = settings["wreckedShipReserve"] && vars.watchers["roomID"].Current == vars.roomIDEnum["bowling"] && (vars.watchers["wreckedShipItems"].Old + 2) == (vars.watchers["wreckedShipItems"].Current);
    var maridiaReserve = settings["maridiaReserve"] && vars.watchers["roomID"].Current == vars.roomIDEnum["leftSandPit"] && (vars.watchers["maridiaItems2"].Old + 2) == (vars.watchers["maridiaItems2"].Current);
    var energyUpgrade = firstETank || allETanks || gauntletETank || terminatorETank || ceilingETank || etecoonsETank || waterwayETank || waveGateETank || kraidETank || crocomireETank || hiJumpETank || ridleyETank || firefleaETank || wreckedShipETank || tatoriETank || botwoonETank || reserveTanks || brinstarReserve || norfairReserve || wreckedShipReserve || maridiaReserve;
    
    // Miniboss room transitions
    var miniBossRooms = false;
    if(settings["miniBossRooms"]){
        var ceresRidleyRoom = vars.watchers["roomID"].Old == vars.roomIDEnum["flatRoom"] && vars.watchers["roomID"].Current == vars.roomIDEnum["ceresRidley"];
        var sporeSpawnRoom = vars.watchers["roomID"].Old == vars.roomIDEnum["sporeSpawnKeyhunter"] && vars.watchers["roomID"].Current == vars.roomIDEnum["sporeSpawn"];
        var crocomireRoom = vars.watchers["roomID"].Old == vars.roomIDEnum["crocomireSpeedway"] && vars.watchers["roomID"].Current == vars.roomIDEnum["crocomire"];
        var botwoonRoom = vars.watchers["roomID"].Old == vars.roomIDEnum["botwoonHallway"] && vars.watchers["roomID"].Current == vars.roomIDEnum["botwoon"];
        // Allow either vanilla or GGG entry
        var goldenTorizoRoom = (vars.watchers["roomID"].Old == vars.roomIDEnum["acidStatue"] || vars.watchers["roomID"].Old == vars.roomIDEnum["screwAttack"]) && vars.watchers["roomID"].Current == vars.roomIDEnum["goldenTorizo"];
        miniBossRooms = ceresRidleyRoom || sporeSpawnRoom || crocomireRoom || botwoonRoom || goldenTorizoRoom;
    }

    // Boss room transitions
    var bossRooms = false;
    if(settings["bossRooms"]){
        var kraidRoom = vars.watchers["roomID"].Old == vars.roomIDEnum["kraidEyeDoor"] && vars.watchers["roomID"].Current == vars.roomIDEnum["kraid"];
        var phantoonRoom = vars.watchers["roomID"].Old == vars.roomIDEnum["basement"] && vars.watchers["roomID"].Current == vars.roomIDEnum["phantoon"];
        var draygonRoom = vars.watchers["roomID"].Old == vars.roomIDEnum["precious"] && vars.watchers["roomID"].Current == vars.roomIDEnum["draygon"];
        var ridleyRoom = vars.watchers["roomID"].Old == vars.roomIDEnum["lowerNorfairFarming"] && vars.watchers["roomID"].Current == vars.roomIDEnum["ridley"];
        var motherBrainRoom = vars.watchers["roomID"].Old == vars.roomIDEnum["rinkaShaft"] && vars.watchers["roomID"].Current == vars.roomIDEnum["motherBrain"];
        bossRooms = kraidRoom || phantoonRoom || draygonRoom || ridleyRoom || motherBrainRoom;
    }

    // Elevator transitions between areas
    var elevatorTransitions = false;
    if(settings["elevatorTransitions"]){
        var blueBrinstar = (vars.watchers["roomID"].Old == vars.roomIDEnum["elevatorToMorphBall"] && vars.watchers["roomID"].Current == vars.roomIDEnum["morphBall"]) || (vars.watchers["roomID"].Old == vars.roomIDEnum["morphBall"] && vars.watchers["roomID"].Current == vars.roomIDEnum["elevatorToMorphBall"]);
        var greenBrinstar = (vars.watchers["roomID"].Old == vars.roomIDEnum["elevatorToGreenBrinstar"] && vars.watchers["roomID"].Current == vars.roomIDEnum["greenBrinstarMainShaft"]) || (vars.watchers["roomID"].Old == vars.roomIDEnum["greenBrinstarMainShaft"] && vars.watchers["roomID"].Current == vars.roomIDEnum["elevatorToGreenBrinstar"]);
        var businessCenter = (vars.watchers["roomID"].Old == vars.roomIDEnum["warehouseEntrance"] && vars.watchers["roomID"].Current == vars.roomIDEnum["businessCenter"]) || (vars.watchers["roomID"].Old == vars.roomIDEnum["businessCenter"] && vars.watchers["roomID"].Current == vars.roomIDEnum["warehouseEntrance"]);
        var caterpillar = (vars.watchers["roomID"].Old == vars.roomIDEnum["elevatorToCaterpillar"] && vars.watchers["roomID"].Current == vars.roomIDEnum["caterpillar"]) || (vars.watchers["roomID"].Old == vars.roomIDEnum["caterpillar"] && vars.watchers["roomID"].Current == vars.roomIDEnum["elevatorToCaterpillar"]);
        var maridiaElevator = (vars.watchers["roomID"].Old == vars.roomIDEnum["elevatorToMaridia"] && vars.watchers["roomID"].Current == vars.roomIDEnum["maridiaElevator"]) || (vars.watchers["roomID"].Old == vars.roomIDEnum["maridiaElevator"] && vars.watchers["roomID"].Current == vars.roomIDEnum["elevatorToMaridia"]);
        elevatorTransitions = blueBrinstar || greenBrinstar || businessCenter || caterpillar || maridiaElevator;
    }

    // Room transitions
    var ceresEscape = settings["ceresEscape"] && vars.watchers["roomID"].Current == vars.roomIDEnum["ceresElevator"] && vars.watchers["gameState"].Old == vars.gameStateEnum["normalGameplay"] && vars.watchers["gameState"].Current == vars.gameStateEnum["startOfCeresCutscene"];
    var wreckedShipEntrance = settings["wreckedShipEntrance"] && vars.watchers["roomID"].Old == vars.roomIDEnum["westOcean"] && vars.watchers["roomID"].Current == vars.roomIDEnum["wreckedShipEntrance"];
    var redTowerMiddleEntrance = settings["redTowerMiddleEntrance"] && vars.watchers["roomID"].Old == vars.roomIDEnum["noobBridge"] && vars.watchers["roomID"].Current == vars.roomIDEnum["redTower"];
    var redTowerBottomEntrance = settings["redTowerBottomEntrance"] && vars.watchers["roomID"].Old == vars.roomIDEnum["bat"] && vars.watchers["roomID"].Current == vars.roomIDEnum["redTower"];
    var kraidsLair = settings["kraidsLair"] && vars.watchers["roomID"].Old == vars.roomIDEnum["warehouseEntrance"] && vars.watchers["roomID"].Current == vars.roomIDEnum["warehouseZeela"];
    var risingTideEntrance = settings["risingTideEntrance"] && vars.watchers["roomID"].Old == vars.roomIDEnum["cathedral"] && vars.watchers["roomID"].Current == vars.roomIDEnum["risingTide"];
    var atticExit = settings["atticExit"] && vars.watchers["roomID"].Old == vars.roomIDEnum["attic"] && vars.watchers["roomID"].Current == vars.roomIDEnum["westOcean"];
    var tubeBroken = settings["tubeBroken"] && vars.watchers["roomID"].Current == vars.roomIDEnum["glassTunnel"] && (vars.watchers["eventFlags"].Old & vars.eventFlagEnum["tubeBroken"]) == 0 && (vars.watchers["eventFlags"].Current & vars.eventFlagEnum["tubeBroken"]) > 0;
    var cacExit = settings["cacExit"] && vars.watchers["roomID"].Old == vars.roomIDEnum["westCactusAlley"] && vars.watchers["roomID"].Current == vars.roomIDEnum["butterflyRoom"];
    var toilet = settings["toilet"] && (vars.watchers["roomID"].Old == vars.roomIDEnum["plasmaSpark"] && vars.watchers["roomID"].Current == vars.roomIDEnum["toiletBowl"] || vars.watchers["roomID"].Old == vars.roomIDEnum["oasis"] && vars.watchers["roomID"].Current == vars.roomIDEnum["toiletBowl"]);
    var kronicBoost = settings["kronicBoost"] && (vars.watchers["roomID"].Old == vars.roomIDEnum["magdolliteTunnel"] && vars.watchers["roomID"].Current == vars.roomIDEnum["kronicBoost"] || vars.watchers["roomID"].Old == vars.roomIDEnum["spikyAcidSnakes"] && vars.watchers["roomID"].Current == vars.roomIDEnum["kronicBoost"] || vars.watchers["roomID"].Old == vars.roomIDEnum["volcano"] && vars.watchers["roomID"].Current == vars.roomIDEnum["kronicBoost"]);
    var lowerNorfairEntrance = settings["lowerNorfairEntrance"] && vars.watchers["roomID"].Old == vars.roomIDEnum["lowerNorfairElevator"] && vars.watchers["roomID"].Current == vars.roomIDEnum["mainHall"];
    var writg = settings["writg"] && vars.watchers["roomID"].Old == vars.roomIDEnum["pillars"] && vars.watchers["roomID"].Current == vars.roomIDEnum["writg"];
    var redKiShaft = settings["redKiShaft"] && (vars.watchers["roomID"].Old == vars.roomIDEnum["amphitheatre"] && vars.watchers["roomID"].Current == vars.roomIDEnum["redKiShaft"] || vars.watchers["roomID"].Old == vars.roomIDEnum["wasteland"] && vars.watchers["roomID"].Current == vars.roomIDEnum["redKiShaft"]);
    var metalPirates = settings["metalPirates"] && vars.watchers["roomID"].Old == vars.roomIDEnum["wasteland"] && vars.watchers["roomID"].Current == vars.roomIDEnum["metalPirates"];
    var lowerNorfairSpringMaze = settings["lowerNorfairSpringMaze"] && vars.watchers["roomID"].Old == vars.roomIDEnum["lowerNorfairFireflea"] && vars.watchers["roomID"].Current == vars.roomIDEnum["lowerNorfairSpringMaze"];
    var lowerNorfairExit = settings["lowerNorfairExit"] && vars.watchers["roomID"].Old == vars.roomIDEnum["threeMusketeers"] && vars.watchers["roomID"].Current == vars.roomIDEnum["singleChamber"];
    var allBossesFinished = (vars.watchers["brinstarBosses"].Current & vars.bossFlagEnum["kraid"]) > 0 && (vars.watchers["wreckedShipBosses"].Current & vars.bossFlagEnum["phantoon"]) > 0 && (vars.watchers["maridiaBosses"].Current & vars.bossFlagEnum["draygon"]) > 0 && (vars.watchers["norfairBosses"].Current & vars.bossFlagEnum["ridley"]) > 0;
    var goldenFour = settings["goldenFour"] && vars.watchers["roomID"].Old == vars.roomIDEnum["statuesHallway"] && vars.watchers["roomID"].Current == vars.roomIDEnum["statues"] && allBossesFinished;
    var tourianEntrance = settings["tourianEntrance"] && vars.watchers["roomID"].Old == vars.roomIDEnum["statues"] && vars.watchers["roomID"].Current == vars.roomIDEnum["tourianElevator"];
    var metroids = settings["metroids"] && (vars.watchers["roomID"].Old == vars.roomIDEnum["metroidOne"] && vars.watchers["roomID"].Current == vars.roomIDEnum["metroidTwo"] || vars.watchers["roomID"].Old == vars.roomIDEnum["metroidTwo"] && vars.watchers["roomID"].Current == vars.roomIDEnum["metroidThree"] || vars.watchers["roomID"].Old == vars.roomIDEnum["metroidThree"] && vars.watchers["roomID"].Current == vars.roomIDEnum["metroidFour"] || vars.watchers["roomID"].Old == vars.roomIDEnum["metroidFour"] && vars.watchers["roomID"].Current == vars.roomIDEnum["tourianHopper"]);
    var babyMetroidRoom = settings["babyMetroidRoom"] && vars.watchers["roomID"].Old == vars.roomIDEnum["dustTorizo"] && vars.watchers["roomID"].Current == vars.roomIDEnum["bigBoy"];
    var escapeClimb = settings["escapeClimb"] && vars.watchers["roomID"].Old == vars.roomIDEnum["tourianEscape4"] && vars.watchers["roomID"].Current == vars.roomIDEnum["climb"];
    var roomTransitions = miniBossRooms || bossRooms || elevatorTransitions || ceresEscape || wreckedShipEntrance || redTowerMiddleEntrance || redTowerBottomEntrance || kraidsLair || risingTideEntrance || atticExit || tubeBroken || cacExit || toilet || kronicBoost || lowerNorfairEntrance || writg || redKiShaft || metalPirates || lowerNorfairSpringMaze || lowerNorfairExit || tourianEntrance || goldenFour || metroids || babyMetroidRoom || escapeClimb;

    // Minibosses
    var ceresRidley = settings["ceresRidley"] && (vars.watchers["ceresBosses"].Old & vars.bossFlagEnum["ceresRidley"]) == 0 && (vars.watchers["ceresBosses"].Current & vars.bossFlagEnum["ceresRidley"]) > 0 && vars.watchers["roomID"].Current == vars.roomIDEnum["ceresRidley"];
    var bombTorizo = settings["bombTorizo"] && (vars.watchers["crateriaBosses"].Old & vars.bossFlagEnum["bombTorizo"]) == 0 && (vars.watchers["crateriaBosses"].Current & vars.bossFlagEnum["bombTorizo"]) > 0 && vars.watchers["roomID"].Current == vars.roomIDEnum["bombTorizo"];
    var sporeSpawn = settings["sporeSpawn"] && (vars.watchers["brinstarBosses"].Old & vars.bossFlagEnum["sporeSpawn"]) == 0 && (vars.watchers["brinstarBosses"].Current & vars.bossFlagEnum["sporeSpawn"]) > 0 && vars.watchers["roomID"].Current == vars.roomIDEnum["sporeSpawn"];
    var crocomire = settings["crocomire"] && (vars.watchers["norfairBosses"].Old & vars.bossFlagEnum["crocomire"]) == 0 && (vars.watchers["norfairBosses"].Current & vars.bossFlagEnum["crocomire"]) > 0 && vars.watchers["roomID"].Current == vars.roomIDEnum["crocomire"];
    var botwoon = settings["botwoon"] && (vars.watchers["maridiaBosses"].Old & vars.bossFlagEnum["botwoon"]) == 0 && (vars.watchers["maridiaBosses"].Current & vars.bossFlagEnum["botwoon"]) > 0 && vars.watchers["roomID"].Current == vars.roomIDEnum["botwoon"];
    var goldenTorizo = settings["goldenTorizo"] && (vars.watchers["norfairBosses"].Old & vars.bossFlagEnum["goldenTorizo"]) == 0 && (vars.watchers["norfairBosses"].Current & vars.bossFlagEnum["goldenTorizo"]) > 0 && vars.watchers["roomID"].Current == vars.roomIDEnum["goldenTorizo"];
    var minibossDefeat = ceresRidley || bombTorizo || sporeSpawn || crocomire || botwoon || goldenTorizo;

    // Bosses
    var kraid = settings["kraid"] && (vars.watchers["brinstarBosses"].Old & vars.bossFlagEnum["kraid"]) == 0 && (vars.watchers["brinstarBosses"].Current & vars.bossFlagEnum["kraid"]) > 0 && vars.watchers["roomID"].Current == vars.roomIDEnum["kraid"];
    if(kraid){
        vars.DebugOutput("Split due to kraid defeat");
    }
    var phantoon = settings["phantoon"] && (vars.watchers["wreckedShipBosses"].Old & vars.bossFlagEnum["phantoon"]) == 0 && (vars.watchers["wreckedShipBosses"].Current & vars.bossFlagEnum["phantoon"]) > 0 && vars.watchers["roomID"].Current == vars.roomIDEnum["phantoon"];
    if(phantoon){
        vars.DebugOutput("Split due to phantoon defeat");
    }
    var draygon = settings["draygon"] && (vars.watchers["maridiaBosses"].Old & vars.bossFlagEnum["draygon"]) == 0 && (vars.watchers["maridiaBosses"].Current & vars.bossFlagEnum["draygon"]) > 0 && vars.watchers["roomID"].Current == vars.roomIDEnum["draygon"];
    if(draygon){
        vars.DebugOutput("Split due to draygon defeat");
    }
    var ridley = settings["ridley"] && (vars.watchers["norfairBosses"].Old & vars.bossFlagEnum["ridley"]) == 0 && (vars.watchers["norfairBosses"].Current & vars.bossFlagEnum["ridley"]) > 0 && vars.watchers["roomID"].Current == vars.roomIDEnum["ridley"];
    if(ridley){
        vars.DebugOutput("Split due to ridley defeat");
    }
    // Mother Brain phases
    var inMotherBrainRoom = vars.watchers["roomID"].Current == vars.roomIDEnum["motherBrain"];
    var mb1 = settings["mb1"] && inMotherBrainRoom && vars.watchers["gameState"].Current == vars.gameStateEnum["normalGameplay"] && vars.watchers["motherBrainHP"].Old == 0 && vars.watchers["motherBrainHP"].Current == (vars.motherBrainMaxHPEnum["phase2"]);
    if(mb1){
        vars.DebugOutput("Split due to mb1 defeat");
    }
    var mb2 = settings["mb2"] && inMotherBrainRoom && vars.watchers["gameState"].Current == vars.gameStateEnum["normalGameplay"] && vars.watchers["motherBrainHP"].Old == 0 && vars.watchers["motherBrainHP"].Current == (vars.motherBrainMaxHPEnum["phase3"]);
    if(mb2){
        vars.DebugOutput("Split due to mb2 defeat");
    }
    var mb3 = settings["mb3"] && inMotherBrainRoom && (vars.watchers["tourianBosses"].Old & vars.bossFlagEnum["motherBrain"]) == 0 && (vars.watchers["tourianBosses"].Current & vars.bossFlagEnum["motherBrain"]) > 0;
    if(mb3){
        vars.DebugOutput("Split due to mb3 defeat");
    }
    var bossDefeat = kraid || phantoon || draygon || ridley || mb1 || mb2 || mb3;

    // Run-ending splits
    var escape = settings["rtaFinish"] && (vars.watchers["eventFlags"].Current & vars.eventFlagEnum["zebesAblaze"]) > 0 && vars.watchers["shipAI"].Old != 0xaa4f && vars.watchers["shipAI"].Current == 0xaa4f;

    var takeoff = settings["igtFinish"] && vars.watchers["roomID"].Current == vars.roomIDEnum["landingSite"] && vars.watchers["gameState"].Old == vars.gameStateEnum["preEndCutscene"] && vars.watchers["gameState"].Current == vars.gameStateEnum["endCutscene"];

    var sporeSpawnRTAFinish = false;
    if(settings["sporeSpawnRTAFinish"]){
        if(vars.pickedUpSporeSpawnSuper){
            if(vars.watchers["igtFrames"].Old != vars.watchers["igtFrames"].Current){
                sporeSpawnRTAFinish = true;
                vars.pickedUpSporeSpawnSuper = false;
            }
        }
        else {
            vars.pickedUpSporeSpawnSuper = vars.watchers["roomID"].Current == vars.roomIDEnum["sporeSpawnSuper"] && (vars.watchers["maxSupers"].Old + 5) == (vars.watchers["maxSupers"].Current) && (vars.watchers["brinstarBosses"].Current & vars.bossFlagEnum["sporeSpawn"]) > 0;
        }
    }

    var hundredMissileRTAFinish = false;
    if(settings["hundredMissileRTAFinish"]){
        if(vars.pickedUpHundredthMissile){
            if(vars.watchers["igtFrames"].Old != vars.watchers["igtFrames"].Current){
                hundredMissileRTAFinish = true;
                vars.pickedUpHundredthMissile = false;
            }
        }
        else{
            vars.pickedUpHundredthMissile = vars.watchers["maxMissiles"].Old == 95 && vars.watchers["maxMissiles"].Current == 100;
        }
    }

    var nonStandardCategoryFinish = sporeSpawnRTAFinish || hundredMissileRTAFinish;

    if(pickup){
        vars.DebugOutput("Split due to pickup");
    }
    if(unlock){
        vars.DebugOutput("Split due to unlock");
    }
    if(beam){
        vars.DebugOutput("Split due to beam upgrade");
    }
    if(energyUpgrade){
        vars.DebugOutput("Split due to energy upgrade");
    }
    if(roomTransitions){
        vars.DebugOutput("Split due to room transition");
    }
    if(minibossDefeat){
        vars.DebugOutput("Split due to miniboss defeat");
    }
    // individual boss defeat conditions already covered above
    if(escape){
        vars.DebugOutput("Split due to escape");
    }
    if(takeoff){
        vars.DebugOutput("Split due to takeoff");
    }
    if(nonStandardCategoryFinish){
        vars.DebugOutput("Split due to non standard category finish");
    }

    return pickup || unlock || beam || energyUpgrade || roomTransitions || minibossDefeat || bossDefeat || escape || takeoff || nonStandardCategoryFinish;
}

gameTime
{
    var frames  = vars.watchers["igtFrames"].Current;
    var seconds = vars.watchers["igtSeconds"].Current;
    var minutes = vars.watchers["igtMinutes"].Current;
    var hours   = vars.watchers["igtHours"].Current;

    if(frames == 0 && vars.watchers["igtFrames"].Old == 49){
        vars.frameRate = 50.0;
    }

    current.totalTime = (frames / vars.frameRate) + seconds + (60 * minutes) + (60 * 60 * hours);
    return TimeSpan.FromSeconds(current.totalTime);
}

isLoading
{
    // From the AutoSplit documentation:
    // "If you want the Game Time to not run in between the synchronization interval and only ever return
    // the actual Game Time of the game, make sure to implement isLoading with a constant
    // return value of true."
    return true;
}

