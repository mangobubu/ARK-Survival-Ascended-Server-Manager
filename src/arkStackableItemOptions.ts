import type { ItemStackOption } from './types'

// 数据来源：ARK Wiki Cargo 表 Items（https://ark.wiki.gg/wiki/Special:CargoTables/Items）
// 生成过滤条件：Blueprint 非空且 stackSize > 1；ItemClassString 由 Blueprint 对象名转换为 *_C。
// 用途：ASA Game.ini 的 ConfigOverrideItemMaxQuantity 单物品堆叠覆盖下拉选项。
export const arkStackableItemOptions = [
  {
    "label": "Absorbent Substrate",
    "classString": "PrimalItemResource_SubstrateAbsorbent_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Achatina Paste",
    "classString": "PrimalItemResource_SnailPaste_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Acrocanthosaurus Egg",
    "classString": "PrimalItemConsumable_Egg_Acro_C",
    "category": "Items",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Ceiling",
    "classString": "PrimalItemStructure_AdobeCeiling_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Dinosaur Gate",
    "classString": "PrimalItemStructure_AdobeGateDoor_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Dinosaur Gateway",
    "classString": "PrimalItemStructure_AdobeFrameGate_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Door",
    "classString": "PrimalItemStructure_AdobeDoor_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Doorframe",
    "classString": "PrimalItemStructure_AdobeWallWithDoor_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Double Door",
    "classString": "PrimalItemStructure_DoubleDoor_Adobe_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Double Doorframe",
    "classString": "PrimalItemStructure_DoubleDoorframe_Adobe_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Fence Foundation",
    "classString": "PrimalItemStructure_AdobeFenceFoundation_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Fence Support",
    "classString": "PrimalItemStructure_FenceSupport_Adobe_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Foundation",
    "classString": "PrimalItemStructure_AdobeFloor_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Hatchframe",
    "classString": "PrimalItemStructure_AdobeCeilingWithTrapdoor_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Ladder",
    "classString": "PrimalItemStructure_AdobeLader_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Pillar",
    "classString": "PrimalItemStructure_AdobePillar_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Railing",
    "classString": "PrimalItemStructure_AdobeRailing_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Ramp",
    "classString": "PrimalItemStructure_AdobeRamp_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Staircase",
    "classString": "PrimalItemStructure_AdobeStaircase_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Stairs",
    "classString": "PrimalItemStructure_Ramp_Adobe_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Trapdoor",
    "classString": "PrimalItemStructure_AdobeTrapdoor_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Triangle Ceiling",
    "classString": "PrimalItemStructure_TriCeiling_Adobe_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Triangle Foundation",
    "classString": "PrimalItemStructure_TriFoundation_Adobe_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Triangle Roof",
    "classString": "PrimalItemStructure_TriRoof_Adobe_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Wall",
    "classString": "PrimalItemStructure_AdobeWall_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Window",
    "classString": "PrimalItemStructure_AdobeWindow_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Adobe Windowframe",
    "classString": "PrimalItemStructure_AdobeWallWithWindow_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Advanced Bullet",
    "classString": "PrimalItemAmmo_AdvancedBullet_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Advanced Rifle Bullet",
    "classString": "PrimalItemAmmo_AdvancedRifleBullet_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Advanced Sniper Bullet",
    "classString": "PrimalItemAmmo_AdvancedSniperBullet_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Advanced Workbench (Primitive Plus)",
    "classString": "PrimalItemStructure_AdvancedWorkbench_C",
    "category": "Crafting stations‎",
    "defaultStackSize": 100
  },
  {
    "label": "Afro Head Hair Style",
    "classString": "PrimalItemConsumable_UnlockHair_Head_Afro_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Aggeravic Mushroom",
    "classString": "PrimalItemResource_CommonMushroom_C",
    "category": "Fungi",
    "defaultStackSize": 100
  },
  {
    "label": "Air Drums Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_AirDrum_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Air Guitar Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_AirGuitar_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Alchemy Table (Primitive Plus)",
    "classString": "PrimalItemStructure_AlchemyTable_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Allosaurus Brain",
    "classString": "PrimalItemResource_ApexDrop_Allo_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Allosaurus Egg",
    "classString": "PrimalItemConsumable_Egg_Allo_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Basilisk Fang",
    "classString": "PrimalItemResource_ApexDrop_Basilisk_Alpha_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Carnotaurus Arm",
    "classString": "PrimalItemResource_ApexDrop_AlphaCarno_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Crystal Talon",
    "classString": "PrimalItemResource_ApexDrop_AlphaCrystalWyvern_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Karkinos Claw",
    "classString": "PrimalItemResource_ApexDrop_CrabClaw_C",
    "category": "Items",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Leedsichthys Blubber",
    "classString": "PrimalItemResource_ApexDrop_AlphaLeeds_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Megalodon Fin",
    "classString": "PrimalItemResource_ApexDrop_AlphaMegalodon_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Mosasaur Tooth",
    "classString": "PrimalItemResource_ApexDrop_AlphaMosasaur_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Raptor Claw",
    "classString": "PrimalItemResource_ApexDrop_AlphaRaptor_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Reaper King Barb",
    "classString": "PrimalItemResource_ApexDrop_ReaperBarb_C",
    "category": "Items",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Tusoteuthis Eye",
    "classString": "PrimalItemResource_ApexDrop_AlphaTuso_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Tyrannosaur Tooth",
    "classString": "PrimalItemResource_ApexDrop_AlphaRex_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Alpha Water Talon",
    "classString": "PrimalItemResource_ApexDrop_AlphaWaterWyvern_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Amarberry",
    "classString": "PrimalItemConsumable_Berry_Amarberry_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Amarberry Juice (Primitive Plus)",
    "classString": "PrimalItemConsumable_Smoothie_Armarberry_C",
    "category": "Consumable",
    "defaultStackSize": 10
  },
  {
    "label": "Amarberry Seed",
    "classString": "PrimalItemConsumable_Seed_Amarberry_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Amargasaurus Egg",
    "classString": "PrimalItemConsumable_Egg_Amargasaurus_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Ammo Box",
    "classString": "PrimalItemStructure_AmmoContainer_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Ammonite Bile",
    "classString": "PrimalItemResource_AmmoniteBlood_C",
    "category": "Resources",
    "defaultStackSize": 50
  },
  {
    "label": "AnglerGel",
    "classString": "PrimalItemResource_AnglerGel_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Ankylo Egg",
    "classString": "PrimalItemConsumable_Egg_Ankylo_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "APS Mailbox (Primitive Plus)",
    "classString": "PrimalItemStructure_Mailbox_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Aqualyrium",
    "classString": "PrimalItemResource_Aqualyrium_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Aquatic Mushroom",
    "classString": "PrimalItemConsumable_Mushroom_Aquatic_C",
    "category": "Fungi",
    "defaultStackSize": 100
  },
  {
    "label": "Araneo Egg",
    "classString": "PrimalItemConsumable_Egg_Spider_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Archaeopteryx Egg",
    "classString": "PrimalItemConsumable_Egg_Archa_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Archelon Egg",
    "classString": "PrimalItemConsumable_Egg_Archelon_ASA_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Archer Flex Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Flex2_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Argentavis Egg",
    "classString": "PrimalItemConsumable_Egg_Argent_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Argentavis Talon",
    "classString": "PrimalItemResource_ApexDrop_Argentavis_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "ARK Anniversary Surprise Cake",
    "classString": "PrimalItemStructure_BirthdayCake_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Arthropluera Egg",
    "classString": "PrimalItemConsumable_Egg_Arthro_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Artifact Pedestal",
    "classString": "PrimalItemStructure_TrophyBase_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Ascerbic Mushroom",
    "classString": "PrimalItemConsumable_Mushroom_Ascerbic_C",
    "category": "Fungi",
    "defaultStackSize": 100
  },
  {
    "label": "Aureliax Egg",
    "classString": "PrimalItemConsumable_Egg_SnowDragon_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Auric Mushroom",
    "classString": "PrimalItemConsumable_Mushroom_Auric_C",
    "category": "Fungi",
    "defaultStackSize": 100
  },
  {
    "label": "Azulberry",
    "classString": "PrimalItemConsumable_Berry_Azulberry_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Azulberry Juice (Primitive Plus)",
    "classString": "PrimalItemConsumable_Smoothie_Azulberry_C",
    "category": "Consumable",
    "defaultStackSize": 10
  },
  {
    "label": "Azulberry Seed",
    "classString": "PrimalItemConsumable_Seed_Azulberry_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Backflip Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Backflip_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Baked Bread Loaf (Primitive Plus)",
    "classString": "PrimalItemConsumable_BreadLoaf_C",
    "category": "Consumables",
    "defaultStackSize": 30
  },
  {
    "label": "Baked Honey Loaf (Primitive Plus)",
    "classString": "PrimalItemConsumable_HoneyLoaf_C",
    "category": "Consumable",
    "defaultStackSize": 30
  },
  {
    "label": "Bakers Oven (Primitive Plus)",
    "classString": "PrimalItemStructure_BakersOven_C",
    "category": "Structure",
    "defaultStackSize": 3
  },
  {
    "label": "Barnacle",
    "classString": "PrimalItemResource_Barnacle_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Barrel of Gunpowder (Primitive Plus)",
    "classString": "PrimalItemStructure_Barrel_Gunpowder_C",
    "category": "Explosives",
    "defaultStackSize": 100
  },
  {
    "label": "Barrel Storage",
    "classString": "PrimalItemStructure_StorageBox_Barrel_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Baryonyx Egg",
    "classString": "PrimalItemConsumable_Egg_Baryonyx_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Basic Kibble",
    "classString": "PrimalItemConsumable_Kibble_Base_XSmall_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Basic Maewing Egg",
    "classString": "PrimalItemConsumable_Egg_MilkGlider_XSmall_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Basilisk Scale",
    "classString": "PrimalItemResource_ApexDrop_Basilisk_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Basilosaurus Blubber",
    "classString": "PrimalItemResource_ApexDrop_Basilo_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Battle Tartare",
    "classString": "PrimalItemConsumable_Soup_BattleTartare_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Bear Trap",
    "classString": "PrimalItemStructure_BearTrap_C",
    "category": "Traps",
    "defaultStackSize": 10
  },
  {
    "label": "Beer Barrel",
    "classString": "PrimalItemStructure_BeerBarrel_C",
    "category": "Cooking structures",
    "defaultStackSize": 3
  },
  {
    "label": "Beer Jar",
    "classString": "PrimalItemConsumable_BeerJar_C",
    "category": "Consumables",
    "defaultStackSize": 20
  },
  {
    "label": "Beer Liquid",
    "classString": "PrimalItemResource_Beer_C",
    "category": "Items",
    "defaultStackSize": 100
  },
  {
    "label": "Beeswax (Primitive Plus)",
    "classString": "PrimalItemResource_Beeswax_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Behemoth Adobe Dinosaur Gate",
    "classString": "PrimalItemStructure_AdobeGatedoor_Large_C",
    "category": "Adobe",
    "defaultStackSize": 5
  },
  {
    "label": "Behemoth Adobe Dinosaur Gateway",
    "classString": "PrimalItemStructure_AdobeGateframe_Large_C",
    "category": "Adobe",
    "defaultStackSize": 5
  },
  {
    "label": "Behemoth Gate",
    "classString": "PrimalItemStructure_MetalGate_Large_C",
    "category": "Metal",
    "defaultStackSize": 5
  },
  {
    "label": "Behemoth Gateway",
    "classString": "PrimalItemStructure_MetalGateframe_Large_C",
    "category": "Metal",
    "defaultStackSize": 5
  },
  {
    "label": "Behemoth Reinforced Dinosaur Gate",
    "classString": "PrimalItemStructure_StoneGateLarge_C",
    "category": "Stone",
    "defaultStackSize": 5
  },
  {
    "label": "Behemoth Stone Dinosaur Gateway",
    "classString": "PrimalItemStructure_StoneGateframe_Large_C",
    "category": "Stone",
    "defaultStackSize": 5
  },
  {
    "label": "Behemoth Tek Gate",
    "classString": "PrimalItemStructure_TekGate_Large_C",
    "category": "Tek",
    "defaultStackSize": 5
  },
  {
    "label": "Behemoth Tek Gateway",
    "classString": "PrimalItemStructure_TekGateframe_Large_C",
    "category": "Tek",
    "defaultStackSize": 5
  },
  {
    "label": "Belly Rub Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_BellyRub_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Beyla Relic",
    "classString": "PrimalItemResource_MiniBossDrop_Beyla_C",
    "category": "Trophies",
    "defaultStackSize": 100
  },
  {
    "label": "Bicep Smooch Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Flex1_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Big Bonfire (Primitive Plus)",
    "classString": "PrimalItemStructure_BigBonfire_C",
    "category": "Structure",
    "defaultStackSize": 3
  },
  {
    "label": "Big Campfire (Primitive Plus)",
    "classString": "PrimalItemStructure_BigCampfire_C",
    "category": "Structure",
    "defaultStackSize": 3
  },
  {
    "label": "Big Fence - Post (Primitive Plus)",
    "classString": "PrimalItemStructure_BigFencePost_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Bio Toxin",
    "classString": "PrimalItemConsumable_JellyVenom_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Bio-Grinder",
    "classString": "PrimalItemStructure_BioGrinder_C",
    "category": "tool",
    "defaultStackSize": 5
  },
  {
    "label": "Birthday Candle",
    "classString": "PrimalItemResource_BirthdayCandle_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Biscuit (Primitive Plus)",
    "classString": "PrimalItemConsumable_Biscuit_C",
    "category": "Consumables",
    "defaultStackSize": 50
  },
  {
    "label": "Biscuit Batter (Primitive Plus)",
    "classString": "PrimalItemConsumable_CakeBatter_C",
    "category": "Consumable",
    "defaultStackSize": 200
  },
  {
    "label": "Black Coloring",
    "classString": "PrimalItemDye_Black_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Black Pearl",
    "classString": "PrimalItemResource_BlackPearl_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Blood Pack",
    "classString": "PrimalItemConsumable_BloodPack_C",
    "category": "Consumables",
    "defaultStackSize": 100
  },
  {
    "label": "Bloodstalker Egg",
    "classString": "PrimalItemConsumable_Egg_BogSpider_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Blue Coloring",
    "classString": "PrimalItemDye_Blue_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Blue Crystalized Sap",
    "classString": "PrimalItemResource_BlueSap_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Blue Gem",
    "classString": "PrimalItemResource_Gem_BioLum_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Bola",
    "classString": "PrimalItem_WeaponBola_C",
    "category": "Weapons",
    "defaultStackSize": 10
  },
  {
    "label": "Boomerang",
    "classString": "PrimalItem_WeaponBoomerang_C",
    "category": "Ranged weapons",
    "defaultStackSize": 10
  },
  {
    "label": "Boulder",
    "classString": "PrimalItemAmmo_Boulder_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Braggot Mead (Primitive Plus)",
    "classString": "PrimalItemConsumable_Drink_Mead_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Brazier (Primitive Plus)",
    "classString": "PrimalItemStructure_Brazier_C",
    "category": "Structures",
    "defaultStackSize": 25
  },
  {
    "label": "Bread Dough (Primitive Plus)",
    "classString": "PrimalItemResource_Dough_C",
    "category": "Resource",
    "defaultStackSize": 50
  },
  {
    "label": "Brick (Primitive Plus)",
    "classString": "PrimalItemResource_Brick_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Brick Ceiling (Primitive Plus)",
    "classString": "PrimalItemStructure_ConcreteCeiling_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Brick Coloring",
    "classString": "PrimalItemDye_Brick_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Brick Dinosaur Gate (Primitive Plus)",
    "classString": "PrimalItemStructure_BrickGate_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Brick Dinosaur Gateway (Primitive Plus)",
    "classString": "PrimalItemStructure_BrickGateframe_Large_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Brick Foundation (Primitive Plus)",
    "classString": "PrimalItemStructure_ConcreteFloor_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Brick Wall (Primitive Plus)",
    "classString": "PrimalItemStructure_BrickWall_C",
    "category": "Buildings",
    "defaultStackSize": 100
  },
  {
    "label": "Bronto Egg",
    "classString": "PrimalItemConsumable_Egg_Bronto_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Broth (Primitive Plus)",
    "classString": "PrimalItemConsumable_Broth_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Broth of Atlan",
    "classString": "PrimalItemConsumable_Soup_BrothofAtlan_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Broth of Enlightenment",
    "classString": "PrimalItemConsumable_TheHorn_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Brown Coloring",
    "classString": "PrimalItemDye_Brown_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Bug Repellant",
    "classString": "PrimalItemConsumable_BugRepellant_C",
    "category": "Items",
    "defaultStackSize": 100
  },
  {
    "label": "Bunk Bed",
    "classString": "PrimalItemStructure_Bed_Modern_C",
    "category": "Furniture",
    "defaultStackSize": 3
  },
  {
    "label": "Bunny Egg",
    "classString": "PrimalItemStructure_EasterEgg_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Bunny Hop Dance Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_BunnyHopDance_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "C4 Charge",
    "classString": "PrimalItemC4Ammo_C",
    "category": "Explosives",
    "defaultStackSize": 100
  },
  {
    "label": "Cactus Broth",
    "classString": "PrimalItemConsumable_CactusBuffSoup_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Cactus Sap",
    "classString": "PrimalItemConsumable_CactusSap_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Cake (Primitive Plus)",
    "classString": "PrimalItemConsumable_Meal_Cake_C",
    "category": "Consumable",
    "defaultStackSize": 5
  },
  {
    "label": "Cake Slice",
    "classString": "PrimalItemResource_CakeSlice_C",
    "category": "Items",
    "defaultStackSize": 50
  },
  {
    "label": "Calien Soup",
    "classString": "PrimalItemConsumable_Soup_CalienSoup_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Camellia (Tea) Seed (Primitive Plus)",
    "classString": "PrimalItemConsumable_Seed_Tea_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Camelsaurus Egg",
    "classString": "PrimalItemConsumable_Egg_Camelsaurus_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Campfire",
    "classString": "PrimalItemStructure_Campfire_C",
    "category": "Cooking structures",
    "defaultStackSize": 3
  },
  {
    "label": "Candle (Primitive Plus)",
    "classString": "PrimalItemStructure_Candle_C",
    "category": "Structures",
    "defaultStackSize": 3
  },
  {
    "label": "Candy Throw Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_FE_CandyThrow_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Cane Sugar (Primitive Plus)",
    "classString": "PrimalItemConsumable_Sugar_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Cantaloupe Coloring",
    "classString": "PrimalItemDye_Cantaloupe_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Carbon (Primitive Plus)",
    "classString": "PrimalItemResource_Carbon_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Carcharodontosaurus Egg",
    "classString": "PrimalItemConsumable_Egg_Carcha_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Carno Egg",
    "classString": "PrimalItemConsumable_Egg_Carno_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Caroling Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Caroling_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Cashew (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_FreshCashew_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Cashew Milk (Primitive Plus)",
    "classString": "PrimalItemResource_CashewMilk_C",
    "category": "Consumable",
    "defaultStackSize": 20
  },
  {
    "label": "Cashew Tree (Primitive Plus)",
    "classString": "PrimalItemConsumable_Seed_Cashew_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Cauldron (Primitive Plus)",
    "classString": "PrimalItemStructure_Cauldron_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Ceiling Lamp (Primitive Plus)",
    "classString": "PrimalItemStructure_CeilingLamp_C",
    "category": "Structures",
    "defaultStackSize": 3
  },
  {
    "label": "Cement Mixer (Primitive Plus)",
    "classString": "PrimalItemStructure_CementMixer_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Cementing Paste",
    "classString": "PrimalItemResource_ChitinPaste_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Ceratosaurus Egg",
    "classString": "PrimalItemConsumable_Egg_Cerato_ASA_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Chain Bola",
    "classString": "PrimalItemAmmo_ChainBola_C",
    "category": "Ammunition",
    "defaultStackSize": 20
  },
  {
    "label": "Charcoal",
    "classString": "PrimalItemResource_Charcoal_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Check Yourself Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Valentines_SelfCheck_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Chicken and Rice (Primitive Plus)",
    "classString": "PrimalItemConsumable_Meal_ChickenRice_C",
    "category": "Consumable",
    "defaultStackSize": 5
  },
  {
    "label": "Chitin",
    "classString": "PrimalItemResource_Chitin_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Cianberry",
    "classString": "PrimalItemConsumable_Berry_Cianberry_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Cigarette (Primitive Plus)",
    "classString": "PrimalItemConsumable_Cigarette_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Citronal",
    "classString": "PrimalItemConsumable_Veggie_Citronal_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Citronal Seed",
    "classString": "PrimalItemConsumable_Seed_Citronal_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Clay",
    "classString": "PrimalItemResource_Clay_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Claystone (Primitive Plus)",
    "classString": "PrimalItemResource_Claystone_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Clockface",
    "classString": "PrimalItemStructure_SteampunkClock_C",
    "category": "Structures",
    "defaultStackSize": 5
  },
  {
    "label": "Cluster Grenade",
    "classString": "PrimalItem_WeaponClusterGrenade_C",
    "category": "Explosives",
    "defaultStackSize": 100
  },
  {
    "label": "Coal",
    "classString": "PrimalItemResource_Coal_C",
    "category": "Items",
    "defaultStackSize": 200
  },
  {
    "label": "Coal (Primitive Plus)",
    "classString": "PrimalItemResource_Coal_PP_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Coelacanth Fishing Trophy",
    "classString": "PrimalItemStructure_FishMount_Coel_C",
    "category": "Trophies",
    "defaultStackSize": 100
  },
  {
    "label": "Coffin",
    "classString": "PrimalItemStructure_Coffin_C",
    "category": "Furniture",
    "defaultStackSize": 3
  },
  {
    "label": "Compost Bin",
    "classString": "PrimalItemStructure_CompostBin_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Compy Egg",
    "classString": "PrimalItemConsumable_Egg_Compy_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Condensed Gas",
    "classString": "PrimalItemResource_CondensedGas_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Congealed Gas Ball",
    "classString": "PrimalItemResource_Gas_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Construction Table (Primitive Plus)",
    "classString": "PrimalItemStructure_ConstructionTable_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Cooked Bacon (Primitive Plus)",
    "classString": "PrimalItemConsumable_CookedBacon_C",
    "category": "Consumable",
    "defaultStackSize": 25
  },
  {
    "label": "Cooked Fish Fillet (Primitive Plus)",
    "classString": "PrimalItemConsumable_CookedFillet_C",
    "category": "Consumable",
    "defaultStackSize": 25
  },
  {
    "label": "Cooked Fish Meat",
    "classString": "PrimalItemConsumable_CookedMeat_Fish_C",
    "category": "Meat",
    "defaultStackSize": 50
  },
  {
    "label": "Cooked Lamb Chop",
    "classString": "PrimalItemConsumable_CookedLambChop_C",
    "category": "Meat",
    "defaultStackSize": 30
  },
  {
    "label": "Cooked Meat",
    "classString": "PrimalItemConsumable_CookedMeat_C",
    "category": "Meat",
    "defaultStackSize": 50
  },
  {
    "label": "Cooked Meat Jerky",
    "classString": "PrimalItemConsumable_CookedMeat_Jerky_C",
    "category": "Meat",
    "defaultStackSize": 50
  },
  {
    "label": "Cooked Poultry (Primitive Plus)",
    "classString": "PrimalItemConsumable_CookedPoultry_C",
    "category": "Consumable",
    "defaultStackSize": 50
  },
  {
    "label": "Cooked Prime Fish Meat",
    "classString": "PrimalItemConsumable_CookedPrimeMeat_Fish_C",
    "category": "Meat",
    "defaultStackSize": 30
  },
  {
    "label": "Cooked Prime Meat",
    "classString": "PrimalItemConsumable_CookedPrimeMeat_C",
    "category": "Meat",
    "defaultStackSize": 30
  },
  {
    "label": "Cooked Rice (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_CookedRice_C",
    "category": "Consumables",
    "defaultStackSize": 100
  },
  {
    "label": "Cooked Spare Ribs (Primitive Plus)",
    "classString": "PrimalItemConsumable_CookedRibs_C",
    "category": "Consumable",
    "defaultStackSize": 50
  },
  {
    "label": "Cooked Supreme Fish Meat",
    "classString": "PrimalItemConsumable_CookedPrimeMeat_SupremeFish_C",
    "category": "Meat",
    "defaultStackSize": 30
  },
  {
    "label": "Cookie Dough (Primitive Plus)",
    "classString": "PrimalItemResource_CookieDough_C",
    "category": "Items",
    "defaultStackSize": 50
  },
  {
    "label": "Cookies (Primitive Plus)",
    "classString": "PrimalItemConsumable_Cookies_C",
    "category": "Consumables",
    "defaultStackSize": 60
  },
  {
    "label": "Cooking Pot",
    "classString": "PrimalItemStructure_CookingPot_C",
    "category": "Cooking structures",
    "defaultStackSize": 100
  },
  {
    "label": "Cooking Station (Primitive Plus)",
    "classString": "PrimalItemStructure_CookingStation_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Corn Starch (Primitive Plus)",
    "classString": "PrimalItemConsumable_Cornstarch_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Corrupt Heart",
    "classString": "PrimalItemResource_RareDrop_CorruptHeart_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Corrupted Nodule",
    "classString": "PrimalItemResource_CorruptedPolymer_C",
    "category": "Resources",
    "defaultStackSize": 20
  },
  {
    "label": "Corrupted Wood",
    "classString": "PrimalItemResource_CorruptedWood_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Cryolophosaurus Egg",
    "classString": "PrimalItemConsumable_CryolophosaurusEgg_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Crystal",
    "classString": "PrimalItemResource_Crystal_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Crystal Talon",
    "classString": "PrimalItemResource_ApexDrop_CrystalWyvern_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Crystal Wyvern Queen Flag",
    "classString": "PrimalItemStructure_Flag_CIBoss_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Crystallized Wood",
    "classString": "PrimalItemResource_CrystallizedWood_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Cucumis",
    "classString": "PrimalItemConsumable_Veggie_Cucumis_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Cucumis Seed",
    "classString": "PrimalItemConsumable_Seed_Cucumis_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Cup of Coffee (Primitive Plus)",
    "classString": "PrimalItemConsumable_Soup_CoffeeDrink_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Cup of Malt (Primitive Plus)",
    "classString": "PrimalItemResource_Malt_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Cup of Tea (Primitive Plus)",
    "classString": "PrimalItemConsumable_Soup_TeaDrink_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Cured Bacon (Primitive Plus)",
    "classString": "PrimalItemConsumable_CuredBacon_C",
    "category": "Consumables",
    "defaultStackSize": 25
  },
  {
    "label": "Cyan Coloring",
    "classString": "PrimalItemDye_Cyan_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Daco Sushi",
    "classString": "PrimalItemConsumable_DacoSushi_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Dance Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Dance_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Deadfall Trap (Primitive Plus)",
    "classString": "PrimalItemStructure_DeadfallTrap_C",
    "category": "Structure",
    "defaultStackSize": 10
  },
  {
    "label": "Deathworm Horn",
    "classString": "PrimalItemResource_KeratinSpike_C",
    "category": "Resources",
    "defaultStackSize": 20
  },
  {
    "label": "Decor Box",
    "classString": "PrimalItemStructure_DecorBox_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Deinonychus Egg",
    "classString": "PrimalItemConsumable_Egg_Deinonychus_Fertilized_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Delivery Crate",
    "classString": "PrimalItemStructure_StorageBox_Balloon_C",
    "category": "Tools",
    "defaultStackSize": 100
  },
  {
    "label": "Dilo Egg",
    "classString": "PrimalItemConsumable_Egg_Dilo_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Dimetrodon Egg",
    "classString": "PrimalItemConsumable_Egg_Dimetrodon_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Dimorph Egg",
    "classString": "PrimalItemConsumable_Egg_Dimorph_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Dinghy (Primitive Plus)",
    "classString": "PrimalItemRowBoat_C",
    "category": "Vehicles",
    "defaultStackSize": 2
  },
  {
    "label": "Dino Leash",
    "classString": "PrimalItemStructure_DinoLeash_C",
    "category": "Tools",
    "defaultStackSize": 100
  },
  {
    "label": "Dinopithecus King Flag",
    "classString": "PrimalItemStructure_Flag_BossDinopithecus_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Dinosaur Bone",
    "classString": "PrimalItemResource_ARKBone_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Dinosaur Gate",
    "classString": "PrimalItemStructure_WoodGate_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Dinosaur Gateway",
    "classString": "PrimalItemStructure_WoodGateframe_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Diplo Egg",
    "classString": "PrimalItemConsumable_Egg_Diplo_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Dipping Net (Ammo)",
    "classString": "PrimalItemAmmo_DippingNet_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "DJ Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_DJ_Craftable_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Dodo Egg",
    "classString": "PrimalItemConsumable_Egg_Dodo_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Dragon Flag",
    "classString": "PrimalItemStructure_Flag_Dragon_C",
    "category": "Furniture",
    "defaultStackSize": 100
  },
  {
    "label": "Dread Beard Facial Hair Style",
    "classString": "PrimalItemConsumable_UnlockHair_Facial_Dreadbeard_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Dreadlocks Head Hair Style",
    "classString": "PrimalItemConsumable_UnlockHair_Head_Dreadlocks_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Dreadnoughtus Egg",
    "classString": "PrimalItemConsumable_Egg_Dreadnoughtus_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Dried Barley (Primitive Plus)",
    "classString": "PrimalItemResource_DriedBarley_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Dried Rice (Primitive Plus)",
    "classString": "PrimalItemResource_Veggie_Rice_Dry_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Dried Seaweed",
    "classString": "PrimalItemConsumable_CookedPlant_DriedSeaweed_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Dried Tea Bags (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_DriedTea_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Dried Tobacco (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_DriedTobacco_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Dried Wheat (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_DriedWheat_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Dry Firewood (Primitive Plus)",
    "classString": "PrimalItemResource_DriedFirewood_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Earthworms",
    "classString": "PrimalItemConsumable_EarthWorms_C",
    "category": "Items",
    "defaultStackSize": 100
  },
  {
    "label": "Ectoplasm",
    "classString": "PrimalItemResource_Ectoplasm_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Egg Incubator",
    "classString": "PrimalItemStructure_EggIncubator_C",
    "category": "Structures",
    "defaultStackSize": 5
  },
  {
    "label": "Electrical Cable Intersection",
    "classString": "PrimalItemStructure_PowerCableIntersection_C",
    "category": "Electricity",
    "defaultStackSize": 100
  },
  {
    "label": "Electrical Generator",
    "classString": "PrimalItemStructure_PowerGenerator_C",
    "category": "Electricity",
    "defaultStackSize": 100
  },
  {
    "label": "Electrical Outlet",
    "classString": "PrimalItemStructure_PowerOutlet_C",
    "category": "Electricity",
    "defaultStackSize": 100
  },
  {
    "label": "Electronic Binoculars (Survival Evolved)",
    "classString": "PrimalItem_WeaponElectronicBinoculars_C",
    "category": "Tools",
    "defaultStackSize": 100
  },
  {
    "label": "Electronics",
    "classString": "PrimalItemResource_Electronics_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Element",
    "classString": "PrimalItemResource_Element_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Element Dust",
    "classString": "PrimalItemResource_ElementDust_C",
    "category": "Resources",
    "defaultStackSize": 1000
  },
  {
    "label": "Element Ore",
    "classString": "PrimalItemResource_ElementOre_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Element Shard",
    "classString": "PrimalItemResource_ElementShard_C",
    "category": "Resources",
    "defaultStackSize": 1000
  },
  {
    "label": "Element-Imbued Gasoline",
    "classString": "PrimalItemResource_Gasoline_Super_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Elevator Track",
    "classString": "PrimalItemStructure_ElevatorTrackBase_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Embryo Incubator",
    "classString": "PrimalItemStructure_EmbryoIncubator_C",
    "category": "Structures",
    "defaultStackSize": 5
  },
  {
    "label": "Enduro Stew",
    "classString": "PrimalItemConsumable_Soup_EnduroStew_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Energy Brew",
    "classString": "PrimalItemConsumable_StaminaSoup_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Evil Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Evil_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Exceptional Kibble",
    "classString": "PrimalItemConsumable_Kibble_Base_XLarge_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Exceptional Maewing Egg",
    "classString": "PrimalItemConsumable_Egg_MilkGlider_XLarge_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Extinguisher Grenade",
    "classString": "PrimalItem_AntifireGrenade_C",
    "category": "Explosives",
    "defaultStackSize": 10
  },
  {
    "label": "Extraordinary Kibble",
    "classString": "PrimalItemConsumable_Kibble_Base_Special_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Extraordinary Maewing Egg",
    "classString": "PrimalItemConsumable_Egg_MilkGlider_Special_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Fancy Armchair",
    "classString": "PrimalItemStructure_FancyChair_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Fancy Sofa",
    "classString": "PrimalItemStructure_FancyCouch_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Feathered Stone Arrow (Primitive Plus)",
    "classString": "PrimalItemAmmo_ArrowStone_Feathered_C",
    "category": "Arrows",
    "defaultStackSize": 100
  },
  {
    "label": "Featherlight Egg",
    "classString": "PrimalItemConsumable_Egg_LanternBird_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Feathers (Primitive Plus)",
    "classString": "PrimalItemResource_Feathers_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Feeding Trough",
    "classString": "PrimalItemStructure_FeedingTrough_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Fenrisúlfr Flag",
    "classString": "PrimalItemStructure_Flag_Fjordur_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Fermented Wine (Primitive Plus)",
    "classString": "PrimalItemConsumable_Drink_Wine_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Fermenting Barrel (Primitive Plus)",
    "classString": "PrimalItemStructure_BarrelFermenting_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Fiber",
    "classString": "PrimalItemResource_Fibers_C",
    "category": "Resources",
    "defaultStackSize": 300
  },
  {
    "label": "Fillet and Bread (Primitive Plus)",
    "classString": "PrimalItemConsumable_Meal_FilletBread_C",
    "category": "Consumable",
    "defaultStackSize": 5
  },
  {
    "label": "Firewood Holder (Primitive Plus)",
    "classString": "PrimalItemStructure_StorageShed_Firewood_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Fish Basket",
    "classString": "PrimalItemStructure_FishBasket_C",
    "category": "Structures",
    "defaultStackSize": 5
  },
  {
    "label": "Fish Jerky",
    "classString": "PrimalItemConsumable_CookedMeat_Fish_Jerky_C",
    "category": "Cooking",
    "defaultStackSize": 30
  },
  {
    "label": "Fish Net",
    "classString": "PrimalItem_WeaponFishingNet_C",
    "category": "Tools",
    "defaultStackSize": 10
  },
  {
    "label": "Fish Scale",
    "classString": "PrimalItemResource_FishScale_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Fjordhawk Egg",
    "classString": "PrimalItemConsumable_Egg_Fjordhawk_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Flame Arrow",
    "classString": "PrimalItemAmmo_ArrowFlame_C",
    "category": "Arrows",
    "defaultStackSize": 100
  },
  {
    "label": "Flame Arrow (Primitive Plus)",
    "classString": "PrimalItemAmmo_ArrowFlaming_C",
    "category": "Arrows",
    "defaultStackSize": 100
  },
  {
    "label": "Flamethrower Ammo",
    "classString": "PrimalItemAmmo_Flamethrower_C",
    "category": "Ammunition",
    "defaultStackSize": 30
  },
  {
    "label": "Flaming Spear",
    "classString": "PrimalItem_WeaponSpear_Flame_Gauntlet_C",
    "category": "Weapons",
    "defaultStackSize": 10
  },
  {
    "label": "Flexible Electrical Cable",
    "classString": "PrimalItemStructure_Wire_Flex_C",
    "category": "Electricity",
    "defaultStackSize": 100
  },
  {
    "label": "Flint",
    "classString": "PrimalItemResource_Flint_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Flirty Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Flirt_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Flower Toss Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_FlowerToss_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Focal Chili",
    "classString": "PrimalItemConsumable_Soup_FocalChili_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Food Coma Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_FoodComa_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Forest Coloring",
    "classString": "PrimalItemDye_Forest_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Fragmented Green Gem",
    "classString": "PrimalItemResource_FracturedGem_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Fresh Bacon (Primitive Plus)",
    "classString": "PrimalItemConsumable_FreshBacon_C",
    "category": "Consumables",
    "defaultStackSize": 25
  },
  {
    "label": "Fresh Barley (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_Barley_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Fresh Cement (Primitive Plus)",
    "classString": "PrimalItemResource_Cement_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Fresh Fat (Primitive Plus)",
    "classString": "PrimalItemConsumable_FreshAnimalFat_C",
    "category": "Resource",
    "defaultStackSize": 500
  },
  {
    "label": "Fresh Firewood (Primitive Plus)",
    "classString": "PrimalItemResource_FreshFirewood_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Fresh Fish Fillet (Primitive Plus)",
    "classString": "PrimalItemConsumable_FreshFillet_C",
    "category": "Consumables",
    "defaultStackSize": 20
  },
  {
    "label": "Fresh Poultry (Primitive Plus)",
    "classString": "PrimalItemConsumable_FreshPoultry_C",
    "category": "Consumables",
    "defaultStackSize": 5
  },
  {
    "label": "Fresh Rice (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_FreshRice_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Fresh Sorghum (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_Sorghum_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Fresh Spare Ribs (Primitive Plus)",
    "classString": "PrimalItemConsumable_FreshRibs_C",
    "category": "Consumables",
    "defaultStackSize": 25
  },
  {
    "label": "Fresh Spinach (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_Spinach_C",
    "category": "Plants",
    "defaultStackSize": 200
  },
  {
    "label": "Fresh Sugar Juice Bucket (Primitive Plus)",
    "classString": "PrimalItemResource_SugarJuice_C",
    "category": "Resource",
    "defaultStackSize": 20
  },
  {
    "label": "Fresh Sugar Plant (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_Sugar_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Fresh Tea Leaves (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_Tea_C",
    "category": "Consumable",
    "defaultStackSize": 200
  },
  {
    "label": "Fresh Tobacco (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_Tobacco_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Fresh Wheat (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_Wheat_C",
    "category": "Plants",
    "defaultStackSize": 200
  },
  {
    "label": "Fria Curry",
    "classString": "PrimalItemConsumable_Soup_FriaCurry_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Friendly Gesture Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Gesture_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Frontier Lamp",
    "classString": "PrimalItemStructure_OilLamp_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Fruit Press (Primitive Plus)",
    "classString": "PrimalItemStructure_FruitPress_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Fungal Wood",
    "classString": "PrimalItemResource_FungalWood_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Gallimimus Egg",
    "classString": "PrimalItemConsumable_Egg_Galli_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Gasbags bladder",
    "classString": "PrimalItemResource_ApexDrop_GasBag_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Gasoline",
    "classString": "PrimalItemResource_Gasoline_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Giant Adobe Hatchframe",
    "classString": "PrimalItemStructure_AdobeCeilingWithDoorWay_Giant_C",
    "category": "Adobe",
    "defaultStackSize": 5
  },
  {
    "label": "Giant Adobe Trapdoor",
    "classString": "PrimalItemStructure_AdobeCeilingDoorGiant_C",
    "category": "Adobe",
    "defaultStackSize": 5
  },
  {
    "label": "Giant Metal Hatchframe",
    "classString": "PrimalItemStructure_MetalCeilingWithTrapdoorGiant_C",
    "category": "Metal",
    "defaultStackSize": 5
  },
  {
    "label": "Giant Metal Trapdoor",
    "classString": "PrimalItemStructure_MetalTrapdoorGiant_C",
    "category": "Metal",
    "defaultStackSize": 5
  },
  {
    "label": "Giant Reinforced Trapdoor",
    "classString": "PrimalItemStructure_StoneCeilingDoorGiant_C",
    "category": "Stone",
    "defaultStackSize": 5
  },
  {
    "label": "Giant Stone Hatchframe",
    "classString": "PrimalItemStructure_StoneCeilingWithTrapdoorGiant_C",
    "category": "Stone",
    "defaultStackSize": 5
  },
  {
    "label": "Gift Box",
    "classString": "PrimalItemStructure_StorageBox_ChristmasGift_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Gigadesmodus Egg Sac",
    "classString": "PrimalItemConsumable_Egg_BossBat_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Giganotosaurus Egg",
    "classString": "PrimalItemConsumable_Egg_Gigant_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Giganotosaurus Heart",
    "classString": "PrimalItemResource_ApexDrop_Giga_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Gigantoraptor Egg",
    "classString": "PrimalItemConsumable_Egg_Gigantoraptor_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Gilly Feast",
    "classString": "PrimalItemConsumable_GillyFeast_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Glass (Primitive Plus)",
    "classString": "PrimalItemResource_Glass_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Gloon Egg",
    "classString": "PrimalItemConsumable_Egg_LostChargePet_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Glow Stick",
    "classString": "PrimalItem_GlowStick_C",
    "category": "Tools",
    "defaultStackSize": 20
  },
  {
    "label": "Glowtail Egg",
    "classString": "PrimalItemConsumable_Egg_LanternLizard_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Goatee Facial Hair Style",
    "classString": "PrimalItemConsumable_UnlockHair_Facial_Goatee_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Gorilla Flag",
    "classString": "PrimalItemStructure_Flag_Gorilla_C",
    "category": "Furniture",
    "defaultStackSize": 100
  },
  {
    "label": "Grand Tortugar Egg",
    "classString": "PrimalItemConsumable_Egg_GrandTortugar_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Grape Juice (Primitive Plus)",
    "classString": "PrimalItemResource_GrapeJuice_C",
    "category": "Consumable",
    "defaultStackSize": 20
  },
  {
    "label": "Grapes (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_Grape_C",
    "category": "Plants",
    "defaultStackSize": 200
  },
  {
    "label": "Grappling Hook",
    "classString": "PrimalItemAmmo_GrapplingHook_C",
    "category": "Ammunition",
    "defaultStackSize": 10
  },
  {
    "label": "Gravestone",
    "classString": "PrimalItemStructure_Furniture_Gravestone_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Greater Aberrant Sigil",
    "classString": "PrimalItemResource_LostColony_AberrantSigil_Greater_C",
    "category": "Items",
    "defaultStackSize": 1000
  },
  {
    "label": "Greater Crimson Sigil",
    "classString": "PrimalItemResource_LostColony_CrimsonSigil_Greater_C",
    "category": "Items",
    "defaultStackSize": 1000
  },
  {
    "label": "Green Coloring",
    "classString": "PrimalItemDye_Green_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Green Gem",
    "classString": "PrimalItemResource_Gem_Fertile_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Greenhouse Ceiling",
    "classString": "PrimalItemStructure_GreenhouseCeiling_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Greenhouse Door",
    "classString": "PrimalItemStructure_GreenhouseDoor_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Greenhouse Doorframe",
    "classString": "PrimalItemStructure_GreenhouseWallWithDoor_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Greenhouse Double Door",
    "classString": "PrimalItemStructure_DoubleDoor_Greenhouse_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Greenhouse Double Doorframe",
    "classString": "PrimalItemStructure_DoubleDoorframe_Greenhouse_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Greenhouse Triangle Ceiling",
    "classString": "PrimalItemStructure_TriCeiling_Greenhouse_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Greenhouse Triangle Roof",
    "classString": "PrimalItemStructure_TriRoof_Greenhouse_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Greenhouse Wall",
    "classString": "PrimalItemStructure_GreenhouseWall_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Greenhouse Window",
    "classString": "PrimalItemStructure_GreenhouseWindow_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Grenade",
    "classString": "PrimalItem_WeaponGrenade_C",
    "category": "Explosives",
    "defaultStackSize": 10
  },
  {
    "label": "Griffin Egg",
    "classString": "PrimalItemConsumable_Egg_Griffin_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Ground Cashew (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_GroundCashew_C",
    "category": "Items",
    "defaultStackSize": 100
  },
  {
    "label": "Gunpowder",
    "classString": "PrimalItemResource_Gunpowder_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Handmill (Primitive Plus)",
    "classString": "PrimalItemStructure_Handmill_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Happy Clap Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_HappyClap_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Hardened Steel Ingot",
    "classString": "PrimalItemResource_HardenedSteelIngot_C",
    "category": "Resources",
    "defaultStackSize": 300
  },
  {
    "label": "Hati Relic",
    "classString": "PrimalItemResource_MiniBossDrop_Hati_C",
    "category": "Trophies",
    "defaultStackSize": 100
  },
  {
    "label": "Heart Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Heart_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Hesperornis Egg",
    "classString": "PrimalItemConsumable_Egg_Hesperonis_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Hide",
    "classString": "PrimalItemResource_Hide_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Hide Sleeping Bag",
    "classString": "PrimalItemStructure_SleepingBag_Hide_C",
    "category": "Furniture",
    "defaultStackSize": 3
  },
  {
    "label": "Holiday Kiss Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_HolidayKiss_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Holiday Lights",
    "classString": "PrimalItemStructure_XmasLights_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Holiday Stocking",
    "classString": "PrimalItemStructure_Stocking_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Holiday Tree",
    "classString": "PrimalItemStructure_ChristmasTree_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Howl Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_FE_Howl_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Hula Dance Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_HulaDance_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Human Hair",
    "classString": "PrimalItemResource_Hair_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Hungry Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Hungry_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Ichthyornis Egg",
    "classString": "PrimalItemConsumable_Egg_Ichthyornis_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Icing (Primitive Plus)",
    "classString": "PrimalItemConsumable_Icing_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Iguanodon Egg",
    "classString": "PrimalItemConsumable_Egg_Iguanodon_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Improvised Explosive Device",
    "classString": "PrimalItem_WeaponTripwireC4_C",
    "category": "Explosives",
    "defaultStackSize": 10
  },
  {
    "label": "Inclined Electrical Cable",
    "classString": "PrimalItemStructure_PowerCableIncline_C",
    "category": "Electricity",
    "defaultStackSize": 100
  },
  {
    "label": "Industrial Grill (Primitive Plus)",
    "classString": "PrimalItemStructure_IndustrialGrill_C",
    "category": "Structures",
    "defaultStackSize": 3
  },
  {
    "label": "Industrial Preserving Bin",
    "classString": "PrimalItemStructure_IndustrialPreservinBin_C",
    "category": "Structures",
    "defaultStackSize": 5
  },
  {
    "label": "Infected Blubber",
    "classString": "PrimalItemConsumable_AbyssalMeat_InfectedBlubber_C",
    "category": "Meat",
    "defaultStackSize": 100
  },
  {
    "label": "Infected Fin",
    "classString": "PrimalItemConsumable_AbyssalMeat_InfectedFin_C",
    "category": "Meat",
    "defaultStackSize": 100
  },
  {
    "label": "Infected Liver",
    "classString": "PrimalItemConsumable_AbyssalMeat_InfectedLiver_C",
    "category": "Meat",
    "defaultStackSize": 100
  },
  {
    "label": "Infected Meat",
    "classString": "PrimalItemConsumable_AbyssalMeat_C",
    "category": "Meat",
    "defaultStackSize": 100
  },
  {
    "label": "Infected Scale",
    "classString": "PrimalItemConsumable_AbyssalMeat_InfectedScale_C",
    "category": "Meat",
    "defaultStackSize": 100
  },
  {
    "label": "Infected Stomach",
    "classString": "PrimalItemConsumable_AbyssalMeat_InfectedStomach_C",
    "category": "Meat",
    "defaultStackSize": 100
  },
  {
    "label": "Infected Tooth",
    "classString": "PrimalItemConsumable_AbyssalMeat_InfectedTooth_C",
    "category": "Meat",
    "defaultStackSize": 100
  },
  {
    "label": "Jam of Amarberry (Primitive Plus)",
    "classString": "PrimalItemConsumable_Preserves_Armarberry_C",
    "category": "Consumable",
    "defaultStackSize": 25
  },
  {
    "label": "Jam of Azulberry (Primitive Plus)",
    "classString": "PrimalItemConsumable_Preserves_Azulberry_C",
    "category": "Consumable",
    "defaultStackSize": 25
  },
  {
    "label": "Jam of Mejoberry (Primitive Plus)",
    "classString": "PrimalItemConsumable_Preserves_Mejoberry_C",
    "category": "Consumable",
    "defaultStackSize": 25
  },
  {
    "label": "Jam of Tintoberry (Primitive Plus)",
    "classString": "PrimalItemConsumable_Preserves_Tintoberry_C",
    "category": "Consumable",
    "defaultStackSize": 25
  },
  {
    "label": "Jar of Pitch",
    "classString": "PrimalItemAmmo_Boulder_Fire_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Jolly Jump Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_JollyJump_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Kairuku Egg",
    "classString": "PrimalItemConsumable_Egg_Kairuku_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Kaprosuchus Egg",
    "classString": "PrimalItemConsumable_Egg_Kaprosuchus_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Kentro Egg",
    "classString": "PrimalItemConsumable_Egg_Kentro_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Keratin",
    "classString": "PrimalItemResource_Keratin_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Allosaurus Egg)",
    "classString": "PrimalItemConsumable_Kibble_Allo_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Ankylo Egg)",
    "classString": "PrimalItemConsumable_Kibble_AnkyloEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Archaeopteryx Egg)",
    "classString": "PrimalItemConsumable_Kibble_ArchaEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Argentavis Egg)",
    "classString": "PrimalItemConsumable_Kibble_ArgentEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Baryonyx Egg)",
    "classString": "PrimalItemConsumable_Kibble_BaryonyxEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Bronto Egg)",
    "classString": "PrimalItemConsumable_Kibble_SauroEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Carbonemys Egg)",
    "classString": "PrimalItemConsumable_Kibble_TurtleEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Carno Egg)",
    "classString": "PrimalItemConsumable_Kibble_CarnoEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Compy Egg)",
    "classString": "PrimalItemConsumable_Kibble_Compy_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Dilo Egg)",
    "classString": "PrimalItemConsumable_Kibble_DiloEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Dimetrodon Egg)",
    "classString": "PrimalItemConsumable_Kibble_DimetroEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Dimorph Egg)",
    "classString": "PrimalItemConsumable_Kibble_DimorphEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Diplo Egg)",
    "classString": "PrimalItemConsumable_Kibble_DiploEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Dodo Egg)",
    "classString": "PrimalItemConsumable_Kibble_DodoEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Gallimimus Egg)",
    "classString": "PrimalItemConsumable_Kibble_GalliEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Iguanodon Egg)",
    "classString": "PrimalItemConsumable_Kibble_IguanodonEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Kairuku Egg)",
    "classString": "PrimalItemConsumable_Kibble_KairukuEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Kaprosuchus Egg)",
    "classString": "PrimalItemConsumable_Kibble_KaproEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Lystrosaurus Egg)",
    "classString": "PrimalItemConsumable_Kibble_LystroEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Megalosaurus Egg)",
    "classString": "PrimalItemConsumable_Kibble_MegalosaurusEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Moschops Egg)",
    "classString": "PrimalItemConsumable_Kibble_MoschopsEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Oviraptor Egg)",
    "classString": "PrimalItemConsumable_Kibble_OviraptorEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Pachy Egg)",
    "classString": "PrimalItemConsumable_Kibble_PachyEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Pachyrhino Egg)",
    "classString": "PrimalItemConsumable_Kibble_PachyRhinoEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Parasaur Egg)",
    "classString": "PrimalItemConsumable_Kibble_ParaEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Pelagornis Egg)",
    "classString": "PrimalItemConsumable_Kibble_Pela_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Pteranodon Egg)",
    "classString": "PrimalItemConsumable_Kibble_PteroEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Pulmonoscorpius Egg)",
    "classString": "PrimalItemConsumable_Kibble_ScorpionEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Quetzal Egg)",
    "classString": "PrimalItemConsumable_Kibble_QuetzEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Raptor Egg)",
    "classString": "PrimalItemConsumable_Kibble_RaptorEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Rex Egg)",
    "classString": "PrimalItemConsumable_Kibble_RexEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Sarco Egg)",
    "classString": "PrimalItemConsumable_Kibble_SarcoEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Spino Egg)",
    "classString": "PrimalItemConsumable_Kibble_SpinoEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Stego Egg)",
    "classString": "PrimalItemConsumable_Kibble_StegoEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Tapejara Egg)",
    "classString": "PrimalItemConsumable_Kibble_TapejaraEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Terror Bird Egg)",
    "classString": "PrimalItemConsumable_Kibble_TerrorBirdEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Therizinosaurus Egg)",
    "classString": "PrimalItemConsumable_Kibble_TherizinoEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble (Trike Egg)",
    "classString": "PrimalItemConsumable_Kibble_TrikeEgg_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Kibble Mash",
    "classString": "PrimalItemResource_KibbleMash_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "King Titan Flag",
    "classString": "PrimalItemStructure_Flag_KingKaiju_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Knock Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_FEKnock_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Lamppost",
    "classString": "PrimalItemStructure_Lamppost_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Lantern (Primitive Plus)",
    "classString": "PrimalItemStructure_TradingLantern_C",
    "category": "Structures",
    "defaultStackSize": 3
  },
  {
    "label": "Large Adobe Wall",
    "classString": "PrimalItemStructure_LargeWall_Adobe_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Large Bear Trap",
    "classString": "PrimalItemStructure_BearTrap_Large_C",
    "category": "Traps",
    "defaultStackSize": 10
  },
  {
    "label": "Large Crop Plot",
    "classString": "PrimalItemStructure_CropPlot_Large_C",
    "category": "Farming",
    "defaultStackSize": 100
  },
  {
    "label": "Large Metal Wall",
    "classString": "PrimalItemStructure_LargeWall_Metal_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Large Stone Wall",
    "classString": "PrimalItemStructure_LargeWall_Stone_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Large Storage Box",
    "classString": "PrimalItemStructure_StorageBox_Large_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Large Taxidermy Base",
    "classString": "PrimalItemStructure_TaxidermyBase_Large_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Large Tek Wall",
    "classString": "PrimalItemStructure_LargeWall_Tek_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Large Wood Elevator Platform",
    "classString": "PrimalItemStructure_WoodElevatorPlatform_Large_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Large Wooden Wall",
    "classString": "PrimalItemStructure_LargeWall_Wood_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Lasso",
    "classString": "PrimalItem_WeaponLasso_C",
    "category": "Weapons",
    "defaultStackSize": 10
  },
  {
    "label": "Lasso Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Lasso_Craftable_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Lazarus Chowder",
    "classString": "PrimalItemConsumable_Soup_LazarusChowder_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Leather (Primitive Plus)",
    "classString": "PrimalItemResource_Leather_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Leech Blood",
    "classString": "PrimalItemResource_LeechBlood_C",
    "category": "Resources",
    "defaultStackSize": 50
  },
  {
    "label": "Lesser Antidote",
    "classString": "PrimalItemConsumable_CureLow_C",
    "category": "Consumables",
    "defaultStackSize": 100
  },
  {
    "label": "Lettuce (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_FreshLettuce_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Lettuce Seed (Primitive Plus)",
    "classString": "PrimalItemConsumable_Seed_Lettuce_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Library Storage",
    "classString": "PrimalItemStructure_LibraryStorage_C",
    "category": "Structures",
    "defaultStackSize": 5
  },
  {
    "label": "Limestone (Primitive Plus)",
    "classString": "PrimalItemResource_Limestone_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Loadout Mannequin",
    "classString": "PrimalItemStructure_LoadoutDummy_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Longrass",
    "classString": "PrimalItemConsumable_Veggie_Longrass_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Longrass Seed",
    "classString": "PrimalItemConsumable_Seed_Longrass_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Lost Colony Lights",
    "classString": "PrimalItemStructure_LC_Lights_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber (Primitive Plus)",
    "classString": "PrimalItemResource_Lumber_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Ceiling (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberCeiling_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Dinosaur Gate (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberGate_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Dinosaur Gateway (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberGateframe_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Door (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberDoor_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Doorframe (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberWallWithDoor_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Glass Ceiling (Primitive Plus)",
    "classString": "PrimalItemStructure_GlassCeiling_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Glass Doorframe (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberGlassWallWithDoor_II_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Pillar (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberPillar_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Ramp (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberRamp_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Station (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberWorkshop_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Table (Primitive Plus)",
    "classString": "PrimalItemStructure_Furniture_LumberTable_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Window (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberGlassWindow_II_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Lumber Windowframe (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberWallWithWindow_II_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Lystro Egg",
    "classString": "PrimalItemConsumable_Egg_Lystro_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Magenta Coloring",
    "classString": "PrimalItemDye_ActuallyMagenta_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Magmasaur Egg",
    "classString": "PrimalItemConsumable_Egg_Cherufe_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Manganese",
    "classString": "PrimalItemResource_Manganese_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Manticore Flag",
    "classString": "PrimalItemStructure_Flag_Manticore_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Mantis Egg",
    "classString": "PrimalItemConsumable_Egg_Mantis_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Market Stall (Primitive Plus)",
    "classString": "PrimalItemStructure_MarketStall_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Medical Brew",
    "classString": "PrimalItemConsumable_HealSoup_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Medium Crop Plot",
    "classString": "PrimalItemStructure_CropPlot_Medium_C",
    "category": "Farming",
    "defaultStackSize": 100
  },
  {
    "label": "Medium Taxidermy Base",
    "classString": "PrimalItemStructure_TaxidermyBase_Medium_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Medium Wood Elevator Platform",
    "classString": "PrimalItemStructure_WoodElevatorPlatform_Medium_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Megachelon Egg",
    "classString": "PrimalItemConsumable_Egg_GiantTurtle_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Megalania Egg",
    "classString": "PrimalItemConsumable_Egg_Megalania_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Megalania Toxin",
    "classString": "PrimalItemResource_ApexDrop_Megalania_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Megalodon Jaws Fishing Trophy",
    "classString": "PrimalItemStructure_SharkJawsTrophy_C",
    "category": "Trophies",
    "defaultStackSize": 100
  },
  {
    "label": "Megalodon Tooth",
    "classString": "PrimalItemResource_ApexDrop_Megalodon_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Megalosaurus Egg",
    "classString": "PrimalItemConsumable_Egg_Megalosaurus_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Megaraptor Egg",
    "classString": "PrimalItemConsumable_Egg_ValMegaraptor_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Mejoberry",
    "classString": "PrimalItemConsumable_Berry_Mejoberry_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Mejoberry Juice (Primitive Plus)",
    "classString": "PrimalItemConsumable_Smoothie_Mejoberry_C",
    "category": "Consumable",
    "defaultStackSize": 10
  },
  {
    "label": "Mejoberry Seed",
    "classString": "PrimalItemConsumable_Seed_Mejoberry_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Metal",
    "classString": "PrimalItemResource_Metal_C",
    "category": "Resources",
    "defaultStackSize": 300
  },
  {
    "label": "Metal Billboard",
    "classString": "PrimalItemStructure_MetalSign_Large_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Catwalk",
    "classString": "PrimalItemStructure_MetalCatwalk_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Ceiling",
    "classString": "PrimalItemStructure_MetalCeiling_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Cliff Platform",
    "classString": "PrimalItemStructure_Metal_CliffPlatform_C",
    "category": "Metal",
    "defaultStackSize": 3
  },
  {
    "label": "Metal Dinosaur Gate",
    "classString": "PrimalItemStructure_MetalGate_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Dinosaur Gateway",
    "classString": "PrimalItemStructure_MetalGateframe_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Door",
    "classString": "PrimalItemStructure_MetalDoor_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Doorframe",
    "classString": "PrimalItemStructure_MetalWallWithDoor_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Double Door",
    "classString": "PrimalItemStructure_DoubleDoor_Metal_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Double Doorframe",
    "classString": "PrimalItemStructure_DoubleDoorframe_Metal_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Fence Foundation",
    "classString": "PrimalItemStructure_MetalFenceFoundation_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Fence Support",
    "classString": "PrimalItemStructure_FenceSupport_Metal_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Foundation",
    "classString": "PrimalItemStructure_MetalFloor_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Hatchframe",
    "classString": "PrimalItemStructure_MetalCeilingWithTrapdoor_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Ingot",
    "classString": "PrimalItemResource_MetalIngot_C",
    "category": "Resources",
    "defaultStackSize": 300
  },
  {
    "label": "Metal Irrigation Pipe - Flexible",
    "classString": "PrimalItemStructure_PipeFlex_Metal_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Irrigation Pipe - Inclined",
    "classString": "PrimalItemStructure_MetalPipeIncline_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Irrigation Pipe - Intake",
    "classString": "PrimalItemStructure_MetalPipeIntake_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Irrigation Pipe - Intersection",
    "classString": "PrimalItemStructure_MetalPipeIntersection_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Irrigation Pipe - Straight",
    "classString": "PrimalItemStructure_MetalPipeStraight_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Irrigation Pipe - Tap",
    "classString": "PrimalItemStructure_MetalPipeTap_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Irrigation Pipe - Vertical",
    "classString": "PrimalItemStructure_MetalPipeVertical_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Ladder",
    "classString": "PrimalItemStructure_MetalLadder_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Ocean Platform",
    "classString": "PrimalItemStructure_Metal_OceanPlatform_C",
    "category": "Metal",
    "defaultStackSize": 3
  },
  {
    "label": "Metal Pillar",
    "classString": "PrimalItemStructure_MetalPillar_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Railing",
    "classString": "PrimalItemStructure_MetalRailing_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Ramp",
    "classString": "PrimalItemStructure_MetalRamp_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Sign",
    "classString": "PrimalItemStructure_MetalSign_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Spike Wall",
    "classString": "PrimalItemStructure_MetalSpikeWall_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Staircase",
    "classString": "PrimalItemStructure_MetalStairs_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Stairs",
    "classString": "PrimalItemStructure_Ramp_Metal_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Trapdoor",
    "classString": "PrimalItemStructure_MetalTrapdoor_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Triangle Ceiling",
    "classString": "PrimalItemStructure_TriCeiling_Metal_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Triangle Foundation",
    "classString": "PrimalItemStructure_TriFoundation_Metal_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Triangle Roof",
    "classString": "PrimalItemStructure_TriRoof_Metal_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Wall",
    "classString": "PrimalItemStructure_MetalWall_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Wall Sign",
    "classString": "PrimalItemStructure_MetalSign_Wall_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Water Reservoir",
    "classString": "PrimalItemStructure_WaterTankMetal_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Window",
    "classString": "PrimalItemStructure_MetalWindow_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Metal Windowframe",
    "classString": "PrimalItemStructure_MetalWallWithWindow_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Microraptor Egg",
    "classString": "PrimalItemConsumable_Egg_Microraptor_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Mindwipe Tonic",
    "classString": "PrimalItemConsumableRespecSoup_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Miners Box (Primitive Plus)",
    "classString": "PrimalItemStructure_MinersCart_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Minor Aberrant Sigil",
    "classString": "PrimalItemResource_LostColony_AberrantSigil_Minor_C",
    "category": "Items",
    "defaultStackSize": 1000
  },
  {
    "label": "Minor Crimson Sigil",
    "classString": "PrimalItemResource_LostColony_CrimsonSigil_Minor_C",
    "category": "Items",
    "defaultStackSize": 1000
  },
  {
    "label": "Mirror",
    "classString": "PrimalItemStructure_Mirror_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Mistletoe",
    "classString": "PrimalItemResource_MistleToe_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Mistletoe Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Mistletoe_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Modern Grill (Primitive Plus)",
    "classString": "PrimalItemStructure_PrimitiveGrill_C",
    "category": "Structure",
    "defaultStackSize": 3
  },
  {
    "label": "Modern Hanging Shelf (Primitive Plus)",
    "classString": "PrimalItemStructure_StorageShed_ModernWoodHanging_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Modern Storage Shelf (Fiber) (Primitive Plus)",
    "classString": "PrimalItemStructure_StorageShed_ModernWood_Fiber_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Modern Storage Shelf (Primitive Plus)",
    "classString": "PrimalItemStructure_StorageShed_ModernWood_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Moeder Flag",
    "classString": "PrimalItemStructure_Flag_EelBoss_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Mohawk Head Hair Style",
    "classString": "PrimalItemConsumable_UnlockHair_Head_Mohawk_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Monodon Horn",
    "classString": "PrimalItemResource_ApexDrop_Monodon_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Mortar and Pestle",
    "classString": "PrimalItemStructure_MortarAndPestle_C",
    "category": "Crafting Stations",
    "defaultStackSize": 100
  },
  {
    "label": "Moschops Egg",
    "classString": "PrimalItemConsumable_Egg_Moschops_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Mosh Pit Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_MoshPit_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Moth Egg",
    "classString": "PrimalItemConsumable_Egg_Moth_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Moustache Facial Hair Style",
    "classString": "PrimalItemConsumable_UnlockHair_Facial_Moustache_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Mud Coloring",
    "classString": "PrimalItemDye_Mud_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Multi-Panel Flag",
    "classString": "PrimalItemStructure_Flag_C",
    "category": "Furniture",
    "defaultStackSize": 100
  },
  {
    "label": "Mushroom Brew",
    "classString": "PrimalItemConsumable_Soup_MushroomSoup_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Mutagel",
    "classString": "PrimalItemConsumable_Mutagel_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Mutagen",
    "classString": "PrimalItemConsumable_Mutagen_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Mutagen Ooze",
    "classString": "PrimalItemResource_MutagenOoze_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Mutton Chops Facial Hair Style",
    "classString": "PrimalItemConsumable_UnlockHair_Facial_MuttonChops_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Narcoberry",
    "classString": "PrimalItemConsumable_Berry_Narcoberry_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Narcoberry Seed",
    "classString": "PrimalItemConsumable_Seed_Narcoberry_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Narcotic",
    "classString": "PrimalItemConsumable_Narcotic_C",
    "category": "Consumables",
    "defaultStackSize": 100
  },
  {
    "label": "Natural Yeast (Primitive Plus)",
    "classString": "PrimalItemResource_Yeast_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Navy Coloring",
    "classString": "PrimalItemDye_Navy_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Neophyte Horns",
    "classString": "PrimalItemResource_ApexDrop_Neophyte_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Net Projectile",
    "classString": "PrimalItemAmmo_ArrowNet_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Nutcracker Dance Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_NutcrackerDance_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Obsidian",
    "classString": "PrimalItemResource_Obsidian_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Obsidian Arrow (Primitive Plus)",
    "classString": "PrimalItemAmmo_ArrowObsi_C",
    "category": "Arrows",
    "defaultStackSize": 100
  },
  {
    "label": "Obsidian Spear (Primitive Plus)",
    "classString": "PrimalItem_WeaponObsidianSpear_C",
    "category": "Melee weapons",
    "defaultStackSize": 10
  },
  {
    "label": "Oceans Bounty",
    "classString": "PrimalItemConsumable_Soup_OceansBounty_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Oil",
    "classString": "PrimalItemResource_Oil_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Oil Jar",
    "classString": "PrimalItem_WeaponOilJar_C",
    "category": "Weapon",
    "defaultStackSize": 10
  },
  {
    "label": "Oil Tank (Primitive Plus)",
    "classString": "PrimalItemStructure_OilTank_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Olive Coloring",
    "classString": "PrimalItemDye_Olive_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Omnidirectional Lamppost",
    "classString": "PrimalItemStructure_LamppostOmni_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Onchopristis Blade",
    "classString": "PrimalItemResource_ApexDrop_Onchopristis_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Orange Coloring",
    "classString": "PrimalItemDye_Orange_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Organic Oil (Primitive Plus)",
    "classString": "PrimalItemResource_ProcessedOil_Seed_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Organic Polymer",
    "classString": "PrimalItemResource_Polymer_Organic_C",
    "category": "Resources",
    "defaultStackSize": 20
  },
  {
    "label": "Oryraise",
    "classString": "PrimalItemConsumable_Veggie_Rice_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Oryraise Ball",
    "classString": "PrimalItemConsumable_OryraiseBall_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Oryraise Seed",
    "classString": "PrimalItemConsumable_Seed_Rice_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Oviraptor Egg",
    "classString": "PrimalItemConsumable_Egg_Oviraptor_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Pachycephalosaurus Egg",
    "classString": "PrimalItemConsumable_Egg_Pachy_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Pachyrhino Egg",
    "classString": "PrimalItemConsumable_Egg_Pachyrhino_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Painting Canvas",
    "classString": "PrimalItemStructure_PaintingCanvas_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Palaeoctopus Egg",
    "classString": "PrimalItemConsumable_UnderwaterEgg_Paleoctopus_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Pancakes (Primitive Plus)",
    "classString": "PrimalItemConsumable_Pancake_C",
    "category": "Consumables",
    "defaultStackSize": 90
  },
  {
    "label": "Panic Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Panic_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Parachute",
    "classString": "PrimalItemConsumableBuff_Parachute_C",
    "category": "Tools",
    "defaultStackSize": 20
  },
  {
    "label": "Parasaur Egg",
    "classString": "PrimalItemConsumable_Egg_Para_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Parchment Coloring",
    "classString": "PrimalItemDye_Parchment_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Parrot Egg",
    "classString": "PrimalItemConsumable_Egg_Parrot_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Pegomastax Egg",
    "classString": "PrimalItemConsumable_Egg_Pegomastax_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Pelagornis Egg",
    "classString": "PrimalItemConsumable_Egg_Pela_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Pelt",
    "classString": "PrimalItemResource_Pelt_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Pet Display",
    "classString": "PrimalItemStructure_LostColony_ShoulderPetDisplayStand_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Pheromone Dart",
    "classString": "PrimalItemAmmo_AggroTranqDart_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Phoenix Egg",
    "classString": "PrimalItemConsumable_Egg_Phoenix_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Pink Coloring",
    "classString": "PrimalItemDye_Pink_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Piranha Fishing Trophy",
    "classString": "PrimalItemStructure_FishMount_Piranha_C",
    "category": "Trophies",
    "defaultStackSize": 100
  },
  {
    "label": "Pitchfork (Primitive Plus)",
    "classString": "PrimalItem_WeaponPitchfork_C",
    "category": "Melee weapons",
    "defaultStackSize": 10
  },
  {
    "label": "Pizza (Primitive Plus)",
    "classString": "PrimalItemConsumable_Meal_Pizza_C",
    "category": "Consumables",
    "defaultStackSize": 5
  },
  {
    "label": "Pizza Dough (Primitive Plus)",
    "classString": "PrimalItemResource_PizzaDough_C",
    "category": "Consumables",
    "defaultStackSize": 50
  },
  {
    "label": "Plant Species W Seed",
    "classString": "PrimalItemConsumable_Seed_PlantSpeciesW_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Plant Species X Seed",
    "classString": "PrimalItemConsumable_Seed_DefensePlant_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Plant Species Y Seed",
    "classString": "PrimalItemConsumable_Seed_PlantSpeciesY_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Plant Species Y Trap",
    "classString": "PrimalItemStructure_PlantSpeciesYTrap_C",
    "category": "Traps",
    "defaultStackSize": 10
  },
  {
    "label": "Plant Species Z Fruit",
    "classString": "PrimalItem_PlantSpeciesZ_Grenade_C",
    "category": "Explosives",
    "defaultStackSize": 100
  },
  {
    "label": "Plant Species Z Seed",
    "classString": "PrimalItemConsumable_Seed_PlantSpeciesZ_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Poison Grenade",
    "classString": "PrimalItem_PoisonGrenade_C",
    "category": "Weapons",
    "defaultStackSize": 10
  },
  {
    "label": "Polymer",
    "classString": "PrimalItemResource_Polymer_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Pontoon Bridge (Primitive Plus)",
    "classString": "PrimalItemStructure_PontoonBridge_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Ponytail Head Hair Style",
    "classString": "PrimalItemConsumable_UnlockHair_Head_Ponytail_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Popsicle",
    "classString": "PrimalItemResource_Popsicle_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Portable Rope Ladder",
    "classString": "PrimalItemStructure_PortableLadder_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Preserving Bin",
    "classString": "PrimalItemStructure_PreservingBin_C",
    "category": "Cooking structures",
    "defaultStackSize": 100
  },
  {
    "label": "Preserving Campfire (Primitive Plus)",
    "classString": "PrimalItemStructure_PreservervingCampfire_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Preserving Salt",
    "classString": "PrimalItemResource_PreservingSalt_C",
    "category": "Resources",
    "defaultStackSize": 6
  },
  {
    "label": "Pressure Plate",
    "classString": "PrimalItemStructure_PressurePlate_C",
    "category": "Structures",
    "defaultStackSize": 5
  },
  {
    "label": "Prime Aberrant Sigil",
    "classString": "PrimalItemResource_LostColony_AberrantSigil_Prime_C",
    "category": "Items",
    "defaultStackSize": 1000
  },
  {
    "label": "Prime Crimson Sigil",
    "classString": "PrimalItemResource_LostColony_CrimsonSigil_Prime_C",
    "category": "Items",
    "defaultStackSize": 1000
  },
  {
    "label": "Prime Fish Jerky",
    "classString": "PrimalItemConsumable_CookedPrimeMeat_Fish_Jerky_C",
    "category": "Cooking",
    "defaultStackSize": 30
  },
  {
    "label": "Prime Meat Jerky",
    "classString": "PrimalItemConsumable_CookedPrimeMeat_Jerky_C",
    "category": "Meat",
    "defaultStackSize": 30
  },
  {
    "label": "Prime Salad (Primitive Plus)",
    "classString": "PrimalItemConsumable_Meal_PrimeSalad_C",
    "category": "Consumable",
    "defaultStackSize": 5
  },
  {
    "label": "Propellant",
    "classString": "PrimalItemResource_Propellant_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Proposal Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Proposal_Craftable_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Pteranodon Egg",
    "classString": "PrimalItemConsumable_Egg_Ptero_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Pulmonoscorpius Egg",
    "classString": "PrimalItemConsumable_Egg_Scorpion_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Pumpkin",
    "classString": "PrimalItemStructure_Pumpkin_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Purple Coloring",
    "classString": "PrimalItemDye_Purple_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Quetzal Egg",
    "classString": "PrimalItemConsumable_Egg_Quetz_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Raptor Egg",
    "classString": "PrimalItemConsumable_Egg_Raptor_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Rare Flower",
    "classString": "PrimalItemResource_RareFlower_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Rare Mushroom",
    "classString": "PrimalItemResource_RareMushroom_C",
    "category": "Fungi",
    "defaultStackSize": 100
  },
  {
    "label": "Rave Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Rave_Craftable_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Raw Fish Meat",
    "classString": "PrimalItemConsumable_RawMeat_Fish_C",
    "category": "Meat",
    "defaultStackSize": 40
  },
  {
    "label": "Raw Meat",
    "classString": "PrimalItemConsumable_RawMeat_C",
    "category": "Meat",
    "defaultStackSize": 40
  },
  {
    "label": "Raw Salt",
    "classString": "PrimalItemResource_RawSalt_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Reaper Pheromone Gland",
    "classString": "PrimalItemResource_XenomorphPheromoneGland_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Red Coloring",
    "classString": "PrimalItemDye_Red_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Red Crystalized Sap",
    "classString": "PrimalItemResource_RedSap_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Red Element",
    "classString": "PrimalItemResource_LostColony_RedElement_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Red Gem",
    "classString": "PrimalItemResource_Gem_Element_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Refining Forge",
    "classString": "PrimalItemStructure_Forge_C",
    "category": "Crafting Stations",
    "defaultStackSize": 100
  },
  {
    "label": "Regular Kibble",
    "classString": "PrimalItemConsumable_Kibble_Base_Medium_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Regular Maewing Egg",
    "classString": "PrimalItemConsumable_Egg_MilkGlider_Medium_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Reinforced Dinosaur Gate",
    "classString": "PrimalItemStructure_StoneGate_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Reinforced Double Door",
    "classString": "PrimalItemStructure_DoubleDoor_Stone_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Reinforced Glass Door (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberGlassDoor_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Reinforced Trapdoor",
    "classString": "PrimalItemStructure_StoneTrapdoor_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Reinforced Window",
    "classString": "PrimalItemStructure_StoneWindow_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Reinforced Wooden Door",
    "classString": "PrimalItemStructure_StoneDoor_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Remote Keypad",
    "classString": "PrimalItemStructure_Keypad_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Resin",
    "classString": "PrimalItemResource_Resin_C",
    "category": "Consumables",
    "defaultStackSize": 100
  },
  {
    "label": "Rex Egg",
    "classString": "PrimalItemConsumable_Egg_Rex_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Rhyniognatha Pheromone",
    "classString": "PrimalItemConsumableEatable_RhynioPheromone_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Rock Drake Egg",
    "classString": "PrimalItemConsumable_Egg_RockDrake_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Rock Drake Feather",
    "classString": "PrimalItemResource_ApexDrop_RockDrake_C",
    "category": "Items",
    "defaultStackSize": 100
  },
  {
    "label": "Rockarrot",
    "classString": "PrimalItemConsumable_Veggie_Rockarrot_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Rockarrot Seed",
    "classString": "PrimalItemConsumable_Seed_Rockarrot_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Rocket Homing Missile",
    "classString": "PrimalItemAmmo_RocketHomingMissile_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Rocket Propelled Grenade",
    "classString": "PrimalItemAmmo_Rocket_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Rockwell Final Form Flag",
    "classString": "PrimalItemStructure_Flag_RockwellGen2_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Rockwell Flag",
    "classString": "PrimalItemStructure_Flag_Rockwell_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Romantic Facial Hair Style",
    "classString": "PrimalItemConsumable_UnlockHairstyle_Facial_Romantic_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Romantic Head Hair Style",
    "classString": "PrimalItemConsumable_UnlockHairstyle_Head_Romantic_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Rope Ladder",
    "classString": "PrimalItemStructure_RopeLadder_C",
    "category": "Thatch",
    "defaultStackSize": 100
  },
  {
    "label": "Royalty Coloring",
    "classString": "PrimalItemDye_Royalty_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Runestone",
    "classString": "PrimalItemResource_ApexDrop_Runestone_C",
    "category": "Tributes",
    "defaultStackSize": 200
  },
  {
    "label": "Sabertooth Salmon Fishing Trophy",
    "classString": "PrimalItemStructure_FishMount_Salmon_C",
    "category": "Trophies",
    "defaultStackSize": 100
  },
  {
    "label": "Sack of Flour (Primitive Plus)",
    "classString": "PrimalItemResource_Flour_C",
    "category": "Resource",
    "defaultStackSize": 20
  },
  {
    "label": "Salad (Primitive Plus)",
    "classString": "PrimalItemConsumable_Meal_Salad_C",
    "category": "Consumable",
    "defaultStackSize": 5
  },
  {
    "label": "Salt (Primitive Plus)",
    "classString": "PrimalItemResource_Salt_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Salted Jerky (Primitive Plus)",
    "classString": "PrimalItemConsumable_CookedMeat_SaltedJerky_C",
    "category": "Meat",
    "defaultStackSize": 30
  },
  {
    "label": "Salted Prime Jerky (Primitive Plus)",
    "classString": "PrimalItemConsumable_CookedPrimeMeat_SaltedJerky_C",
    "category": "Meat",
    "defaultStackSize": 30
  },
  {
    "label": "Sand",
    "classString": "PrimalItemResource_Sand_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Santa Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Santa_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Sap",
    "classString": "PrimalItemResource_Sap_C",
    "category": "Resources",
    "defaultStackSize": 30
  },
  {
    "label": "Sarco Egg",
    "classString": "PrimalItemConsumable_Egg_Sarco_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Sarcosuchus Skin",
    "classString": "PrimalItemResource_ApexDrop_Sarco_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Sauropod Vertebra",
    "classString": "PrimalItemResource_ApexDrop_Sauro_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Savoroot",
    "classString": "PrimalItemConsumable_Veggie_Savoroot_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Savoroot Seed",
    "classString": "PrimalItemConsumable_Seed_Savoroot_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Scare Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_FEScare_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Scarecrow",
    "classString": "PrimalItemStructure_Scarecrow_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Scrap Metal",
    "classString": "PrimalItemResource_ScrapMetal_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Scrap Metal Ingot",
    "classString": "PrimalItemResource_ScrapMetalIngot_C",
    "category": "Resources",
    "defaultStackSize": 300
  },
  {
    "label": "Sea Dragon Soup",
    "classString": "PrimalItemConsumable_Soup_SeaDragonSoup_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Seaweed",
    "classString": "PrimalItemResource_Seaweed_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Self Hug Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_SelfHug_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Shadow Steak Saute",
    "classString": "PrimalItemConsumable_Soup_ShadowSteak_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Shag Rug",
    "classString": "PrimalItemStructure_Furniture_Rug_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Shell Fragment",
    "classString": "PrimalItemResource_TurtleShell_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Shiver Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Shiver_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Shocking Tranquilizer Dart",
    "classString": "PrimalItemAmmo_RefinedTranqDart_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Shootable Bottle",
    "classString": "PrimalItemStructure_ShootableBottle_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Silica Pearls",
    "classString": "PrimalItemResource_Silicon_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Silicate",
    "classString": "PrimalItemResource_Silicate_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Silk",
    "classString": "PrimalItemResource_Silk_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Silver Coloring",
    "classString": "PrimalItemDye_Silver_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Simple Bed",
    "classString": "PrimalItemStructure_Bed_Simple_C",
    "category": "Furniture",
    "defaultStackSize": 3
  },
  {
    "label": "Simple Bullet",
    "classString": "PrimalItemAmmo_SimpleBullet_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Simple Hammock (Primitive Plus)",
    "classString": "PrimalItemStructure_Hammock_Simple_C",
    "category": "Structures",
    "defaultStackSize": 3
  },
  {
    "label": "Simple Kibble",
    "classString": "PrimalItemConsumable_Kibble_Base_Small_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Simple Maewing Egg",
    "classString": "PrimalItemConsumable_Egg_MilkGlider_Small_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Simple Rifle Ammo",
    "classString": "PrimalItemAmmo_SimpleRifleBullet_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Simple Shotgun Ammo",
    "classString": "PrimalItemAmmo_SimpleShotgunBullet_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Single Panel Flag",
    "classString": "PrimalItemStructure_Flag_Single_C",
    "category": "Furniture",
    "defaultStackSize": 100
  },
  {
    "label": "Sink (Primitive Plus)",
    "classString": "PrimalItemStructure_MetalSink_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Sinomacrops Egg",
    "classString": "PrimalItemConsumable_Egg_Sinomacrops_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Sit Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Sit_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Sköll Relic",
    "classString": "PrimalItemResource_MiniBossDrop_Skoll_C",
    "category": "Trophies",
    "defaultStackSize": 100
  },
  {
    "label": "Sky Coloring",
    "classString": "PrimalItemDye_Sky_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Slate Coloring",
    "classString": "PrimalItemDye_Slate_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Adobe Roof",
    "classString": "PrimalItemStructure_AdobeRoof_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Adobe Wall Left",
    "classString": "PrimalItemStructure_AdobeWall_Sloped_Left_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Adobe Wall Right",
    "classString": "PrimalItemStructure_AdobeWall_Sloped_Right_C",
    "category": "Adobe",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Brick Roof (Primitive Plus)",
    "classString": "PrimalItemStructure_CementRoof_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Glass Roof (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberGlassRoof_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Greenhouse Roof",
    "classString": "PrimalItemStructure_GreenhouseRoof_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Greenhouse Wall Left",
    "classString": "PrimalItemStructure_GreenhouseWall_Sloped_Left_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Greenhouse Wall Right",
    "classString": "PrimalItemStructure_GreenhouseWall_Sloped_Right_C",
    "category": "Greenhouse",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Lumber Glass Wall Left (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberGlassWall_Sloped_Left_II_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Lumber Glass Wall Right (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberGlassWall_Sloped_Right_II_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Lumber Roof (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberRoof_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Metal Roof",
    "classString": "PrimalItemStructure_MetalRoof_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Metal Wall Left",
    "classString": "PrimalItemStructure_MetalWall_Sloped_Left_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Metal Wall Right",
    "classString": "PrimalItemStructure_MetalWall_Sloped_Right_C",
    "category": "Metal",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Stone Roof",
    "classString": "PrimalItemStructure_StoneRoof_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Stone Wall Left",
    "classString": "PrimalItemStructure_StoneWall_Sloped_Left_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Stone Wall Right",
    "classString": "PrimalItemStructure_StoneWall_Sloped_Right_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Tek Roof",
    "classString": "PrimalItemStructure_TekRoof_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Tek Wall Left",
    "classString": "PrimalItemStructure_TekWall_Sloped_Left_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Tek Wall Right",
    "classString": "PrimalItemStructure_TekWall_Sloped_Right_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Thatch Roof",
    "classString": "PrimalItemStructure_ThatchRoof_C",
    "category": "Thatch",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Thatch Wall Right",
    "classString": "PrimalItemStructure_ThatchWall_Sloped_Right_C",
    "category": "Thatch",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Wood Wall Left",
    "classString": "PrimalItemStructure_WoodWall_Sloped_Left_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Wood Wall Right",
    "classString": "PrimalItemStructure_WoodWall_Sloped_Right_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Sloped Wooden Roof",
    "classString": "PrimalItemStructure_WoodRoof_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Small Crop Plot",
    "classString": "PrimalItemStructure_CropPlot_Small_C",
    "category": "Farming",
    "defaultStackSize": 100
  },
  {
    "label": "Small Taxidermy Base",
    "classString": "PrimalItemStructure_TaxidermyBase_Small_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Small Wood Elevator Platform",
    "classString": "PrimalItemStructure_WoodElevatorPlatform_Small_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Smithy",
    "classString": "PrimalItemStructure_AnvilBench_C",
    "category": "Crafting Stations",
    "defaultStackSize": 100
  },
  {
    "label": "Smoke Grenade",
    "classString": "PrimalItem_GasGrenade_C",
    "category": "Explosives",
    "defaultStackSize": 10
  },
  {
    "label": "Smokehouse (Primitive Plus)",
    "classString": "PrimalItemStructure_Smokehouse_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Sniff Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_FE_Sniff_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Snow Angel Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_SnowAngel_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Snow Owl Egg",
    "classString": "PrimalItemConsumable_Egg_Owl_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Snowball Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Snowball_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Snowman",
    "classString": "PrimalItemStructure_Snowman_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Soap",
    "classString": "PrimalItemConsumableSoap_C",
    "category": "Tools",
    "defaultStackSize": 100
  },
  {
    "label": "Sorghum Seed (Primitive Plus)",
    "classString": "PrimalItemConsumable_Seed_Sorghum_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Sorghum Syrup (Primitive Plus)",
    "classString": "PrimalItemConsumable_Syrup_C",
    "category": "Consumable",
    "defaultStackSize": 200
  },
  {
    "label": "Soybean (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_Soybean_C",
    "category": "Plants",
    "defaultStackSize": 200
  },
  {
    "label": "Soybean Seed (Primitive Plus)",
    "classString": "PrimalItemConsumable_Seed_Soybean_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Soymilk (Primitive Plus)",
    "classString": "PrimalItemResource_SoyMilk_C",
    "category": "Resource",
    "defaultStackSize": 20
  },
  {
    "label": "Sparkpowder",
    "classString": "PrimalItemResource_Sparkpowder_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Spear",
    "classString": "PrimalItem_WeaponSpear_C",
    "category": "Melee weapons",
    "defaultStackSize": 10
  },
  {
    "label": "Spear Bolt",
    "classString": "PrimalItemAmmo_BallistaArrow_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Spider Flag",
    "classString": "PrimalItemStructure_Flag_Spider_C",
    "category": "Furniture",
    "defaultStackSize": 100
  },
  {
    "label": "Spinach Seed (Primitive Plus)",
    "classString": "PrimalItemConsumable_Seed_Spinach_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Spinning Mule (Primitive Plus)",
    "classString": "PrimalItemStructure_SpinningMule_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Spino Egg",
    "classString": "PrimalItemConsumable_Egg_Spino_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Spinosaurus Sail",
    "classString": "PrimalItemResource_ApexDrop_Spino_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Spoiled Meat",
    "classString": "PrimalItemConsumable_SpoiledMeat_C",
    "category": "Meat",
    "defaultStackSize": 100
  },
  {
    "label": "Standing Torch",
    "classString": "PrimalItemStructure_StandingTorch_C",
    "category": "Furniture",
    "defaultStackSize": 3
  },
  {
    "label": "Steam Lights & Lamps",
    "classString": "PrimalItemStructure_SteamLights_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Steel Handrail (Primitive Plus)",
    "classString": "PrimalItemStructure_SteelHandrail_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Steel Ingot (Primitive Plus)",
    "classString": "PrimalItemResource_SteelIngot_C",
    "category": "Resource",
    "defaultStackSize": 200
  },
  {
    "label": "Steel Safebox (Primitive Plus)",
    "classString": "PrimalItemStructure_StorageBox_Safe_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Steel Water Tower (Primitive Plus)",
    "classString": "PrimalItemStructure_WaterTower_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Stego Egg",
    "classString": "PrimalItemConsumable_Egg_Stego_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Steinbjörn Relic",
    "classString": "PrimalItemResource_MiniBossDrop_Stein_C",
    "category": "Trophies",
    "defaultStackSize": 100
  },
  {
    "label": "Sticky Bomb (Primitive Plus)",
    "classString": "PrimalItem_WeaponStickyBomb_C",
    "category": "Explosives",
    "defaultStackSize": 10
  },
  {
    "label": "Stimberry",
    "classString": "PrimalItemConsumable_Berry_Stimberry_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Stimberry Seed",
    "classString": "PrimalItemConsumable_Seed_Stimberry_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Stimbull Juice (Primitive Plus)",
    "classString": "PrimalItemConsumable_Smoothie_Stimbull_C",
    "category": "Consumable",
    "defaultStackSize": 10
  },
  {
    "label": "Stimulant",
    "classString": "PrimalItemConsumable_Stimulant_C",
    "category": "Items",
    "defaultStackSize": 100
  },
  {
    "label": "Stolen Headstone",
    "classString": "PrimalItemStructure_HW_Grave_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Stone",
    "classString": "PrimalItemResource_Stone_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Arrow",
    "classString": "PrimalItemAmmo_ArrowStone_C",
    "category": "Arrows",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Ceiling",
    "classString": "PrimalItemStructure_StoneCeiling_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Cliff Platform",
    "classString": "PrimalItemStructure_Stone_CliffPlatform_C",
    "category": "Wood",
    "defaultStackSize": 3
  },
  {
    "label": "Stone Dinosaur Gateway",
    "classString": "PrimalItemStructure_StoneGateframe_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Doorframe",
    "classString": "PrimalItemStructure_StoneWallWithDoor_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Double Doorframe",
    "classString": "PrimalItemStructure_DoubleDoorframe_Stone_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Fence Foundation",
    "classString": "PrimalItemStructure_StoneFenceFoundation_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Fence Support",
    "classString": "PrimalItemStructure_FenceSupport_Stone_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Fireplace",
    "classString": "PrimalItemStructure_Fireplace_C",
    "category": "Furniture",
    "defaultStackSize": 3
  },
  {
    "label": "Stone Foundation",
    "classString": "PrimalItemStructure_StoneFloor_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Hatchframe",
    "classString": "PrimalItemStructure_StoneCeilingWithTrapdoor_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Irrigation Pipe - Flexible",
    "classString": "PrimalItemStructure_PipeFlex_Stone_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Irrigation Pipe - Inclined",
    "classString": "PrimalItemStructure_StonePipeIncline_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Irrigation Pipe - Intake",
    "classString": "PrimalItemStructure_StonePipeIntake_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Irrigation Pipe - Intersection",
    "classString": "PrimalItemStructure_StonePipeIntersection_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Irrigation Pipe - Straight",
    "classString": "PrimalItemStructure_StonePipeStraight_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Irrigation Pipe - Tap",
    "classString": "PrimalItemStructure_StonePipeTap_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Irrigation Pipe - Vertical",
    "classString": "PrimalItemStructure_StonePipeVertical_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Pillar",
    "classString": "PrimalItemStructure_StonePillar_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Railing",
    "classString": "PrimalItemStructure_StoneRailing_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Staircase",
    "classString": "PrimalItemStructure_StoneStairs_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Stairs",
    "classString": "PrimalItemStructure_Ramp_Stone_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Triangle Ceiling",
    "classString": "PrimalItemStructure_TriCeiling_Stone_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Triangle Foundation",
    "classString": "PrimalItemStructure_TriFoundation_Stone_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Triangle Roof",
    "classString": "PrimalItemStructure_TriRoof_Stone_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Wall",
    "classString": "PrimalItemStructure_StoneWall_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Stone Windowframe",
    "classString": "PrimalItemStructure_StoneWallWithWindow_C",
    "category": "Stone",
    "defaultStackSize": 100
  },
  {
    "label": "Storage Barrel (Primitive Plus)",
    "classString": "PrimalItemStructure_Barrel_Normal_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Storage Box",
    "classString": "PrimalItemStructure_StorageBox_Small_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Straight Electrical Cable",
    "classString": "PrimalItemStructure_PowerCableStraight_C",
    "category": "Electricity",
    "defaultStackSize": 100
  },
  {
    "label": "Sulfur",
    "classString": "PrimalItemResource_Sulfur_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Sun Bathe Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_SunBathe_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Superior Kibble",
    "classString": "PrimalItemConsumable_Kibble_Base_Large_C",
    "category": "Kibbles",
    "defaultStackSize": 100
  },
  {
    "label": "Superior Maewing Egg",
    "classString": "PrimalItemConsumable_Egg_MilkGlider_Large_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Sweet Vegetable Cake",
    "classString": "PrimalItemConsumable_SweetVeggieCake_C",
    "category": "Cooking",
    "defaultStackSize": 10
  },
  {
    "label": "Table Lantern (Primitive Plus)",
    "classString": "PrimalItemStructure_GeneralLantern_C",
    "category": "Structures",
    "defaultStackSize": 3
  },
  {
    "label": "Tail Wiggle Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_TailWiggle_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Tan Coloring",
    "classString": "PrimalItemDye_Tan_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Tangerine Coloring",
    "classString": "PrimalItemDye_Tangerine_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Tanning Rack (Primitive Plus)",
    "classString": "PrimalItemStructure_TanningRack_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Tapejara Egg",
    "classString": "PrimalItemConsumable_Egg_Tapejara_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Teepee (Primitive Plus)",
    "classString": "PrimalItemStructure_Teepee_C",
    "category": "Structures",
    "defaultStackSize": 3
  },
  {
    "label": "Tek Behemoth Cellar Door",
    "classString": "PrimalItemStructure_Ceiling_Door_XL_Tek_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Bridge",
    "classString": "PrimalItemStructure_TekBridge_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Catwalk",
    "classString": "PrimalItemStructure_TekCatwalk_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Ceiling",
    "classString": "PrimalItemStructure_TekCeiling_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Crop Plot",
    "classString": "PrimalItemStructure_CropPlot_Tek_C",
    "category": "Farming",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Dedicated Storage",
    "classString": "PrimalItemStructure_DedicatedStorage_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Dinosaur Gate",
    "classString": "PrimalItemStructure_TekGate_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Dinosaur Gateway",
    "classString": "PrimalItemStructure_TekGateframe_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Door",
    "classString": "PrimalItemStructure_TekDoor_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Doorframe",
    "classString": "PrimalItemStructure_TekWallwithdoor_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Double Door",
    "classString": "PrimalItemStructure_DoubleDoor_Tek_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Double Doorframe",
    "classString": "PrimalItemStructure_DoubleDoorframe_Tek_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Fence Foundation",
    "classString": "PrimalItemStructure_Tekfencefoundation_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Fence Support",
    "classString": "PrimalItemStructure_FenceSupport_Tek_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Floating Foundation",
    "classString": "PrimalItemStructure_TekFloatingFoundation_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Foundation",
    "classString": "PrimalItemStructure_TekFloor_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Gravity Grenade",
    "classString": "PrimalItem_WeaponTekGravityGrenade_C",
    "category": "Explosives",
    "defaultStackSize": 10
  },
  {
    "label": "Tek Grenade",
    "classString": "PrimalItem_TekGrenade_C",
    "category": "Explosives",
    "defaultStackSize": 10
  },
  {
    "label": "Tek Hatchframe",
    "classString": "PrimalItemStructure_TekCeilingWithTrapdoor_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Ladder",
    "classString": "PrimalItemStructure_TekLadder_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Large Cellar Door",
    "classString": "PrimalItemStructure_TekCeilingDoor_Giant_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Light",
    "classString": "PrimalItemStructure_TekLight_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Ocean Platform",
    "classString": "PrimalItemStructure_Tek_OceanPlatform_C",
    "category": "Tek",
    "defaultStackSize": 3
  },
  {
    "label": "Tek Parasaur Egg",
    "classString": "PrimalItemConsumable_Egg_Para_Bionic_C",
    "category": "Tek eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Pillar",
    "classString": "PrimalItemStructure_TekPillar_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Quetzal Egg",
    "classString": "PrimalItemConsumable_Egg_Quetz_Bionic_C",
    "category": "Tek eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Railing",
    "classString": "PrimalItemStructure_Tekrailing_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Ramp",
    "classString": "PrimalItemStructure_TekRamp_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Raptor Egg",
    "classString": "PrimalItemConsumable_Egg_Raptor_Bionic_C",
    "category": "Tek eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Rex Egg",
    "classString": "PrimalItemConsumable_Egg_Rex_Bionic_C",
    "category": "Tek eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Sleeping Pod",
    "classString": "PrimalItemStructure_Bed_Tek_C",
    "category": "Furniture",
    "defaultStackSize": 3
  },
  {
    "label": "Tek Staircase",
    "classString": "PrimalItemStructure_TekStairs_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Stairs",
    "classString": "PrimalItemStructure_Ramp_Tek_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Stego Egg",
    "classString": "PrimalItemConsumable_Egg_Stego_Bionic_C",
    "category": "Tek eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Surveillance Console",
    "classString": "PrimalItemStructure_TekSecurityConsole_C",
    "category": "Tek",
    "defaultStackSize": 5
  },
  {
    "label": "Tek Trapdoor",
    "classString": "PrimalItemStructure_TekTrapdoor_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Triangle Ceiling",
    "classString": "PrimalItemStructure_TriCeiling_Tek_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Triangle Floating Foundation",
    "classString": "PrimalItemStructure_TriFloatingFoundation_Te_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Triangle Foundation",
    "classString": "PrimalItemStructure_TriFoundation_Tek_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Triangle Roof",
    "classString": "PrimalItemStructure_TriRoof_Tek_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Trike Egg",
    "classString": "PrimalItemConsumable_Egg_Trike_Bionic_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Wall",
    "classString": "PrimalItemStructure_TekWall_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Window",
    "classString": "PrimalItemStructure_TekWindow_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Tek Windowframe",
    "classString": "PrimalItemStructure_TekWallWithWindow_C",
    "category": "Tek",
    "defaultStackSize": 100
  },
  {
    "label": "Terror Bird Egg",
    "classString": "PrimalItemConsumable_Egg_TerrorBird_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Thalassian Ammo",
    "classString": "PrimalItemAmmo_ThalassianBullet_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Thatch",
    "classString": "PrimalItemResource_Thatch_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Thatch Ceiling",
    "classString": "PrimalItemStructure_ThatchCeiling_C",
    "category": "Thatch",
    "defaultStackSize": 100
  },
  {
    "label": "Thatch Door",
    "classString": "PrimalItemStructure_ThatchDoor_C",
    "category": "Thatch",
    "defaultStackSize": 100
  },
  {
    "label": "Thatch Doorframe",
    "classString": "PrimalItemStructure_ThatchWallWithDoor_C",
    "category": "Thatch",
    "defaultStackSize": 100
  },
  {
    "label": "Thatch Foundation",
    "classString": "PrimalItemStructure_ThatchFloor_C",
    "category": "Thatch",
    "defaultStackSize": 100
  },
  {
    "label": "Thatch Wall",
    "classString": "PrimalItemStructure_ThatchWall_C",
    "category": "Thatch",
    "defaultStackSize": 100
  },
  {
    "label": "Therizino Claws",
    "classString": "PrimalItemResource_ApexDrop_Theriz_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Therizino Egg",
    "classString": "PrimalItemConsumable_Egg_Therizino_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "This Big Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_ThisBig_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Thorny Dragon Egg",
    "classString": "PrimalItemConsumable_Egg_SpineyLizard_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Thylacoleo Hook-Claw",
    "classString": "PrimalItemResource_ApexDrop_Thylaco_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Tinkering Desk",
    "classString": "PrimalItemStructure_TinkeringDesk_C",
    "category": "Structures",
    "defaultStackSize": 5
  },
  {
    "label": "Tintoberry",
    "classString": "PrimalItemConsumable_Berry_Tintoberry_C",
    "category": "Plants",
    "defaultStackSize": 100
  },
  {
    "label": "Tintoberry Juice (Primitive Plus)",
    "classString": "PrimalItemConsumable_Smoothie_Tintoberry_C",
    "category": "Consumable",
    "defaultStackSize": 10
  },
  {
    "label": "Tintoberry Seed",
    "classString": "PrimalItemConsumable_Seed_Tintoberry_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Titanboa Egg",
    "classString": "PrimalItemConsumable_Egg_Boa_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Titanoboa Venom",
    "classString": "PrimalItemResource_ApexDrop_Boa_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Tobacco Seed (Primitive Plus)",
    "classString": "PrimalItemConsumable_Seed_Tobacco_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Tofu (Primitive Plus)",
    "classString": "PrimalItemResource_Tofu_C",
    "category": "Resource",
    "defaultStackSize": 50
  },
  {
    "label": "Toilet",
    "classString": "PrimalItemStructure_Toilet_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Tomato (Primitive Plus)",
    "classString": "PrimalItemConsumable_Veggie_Tomato_C",
    "category": "Consumable",
    "defaultStackSize": 100
  },
  {
    "label": "Tomato Juice (Primitive Plus)",
    "classString": "PrimalItemConsumable_Smoothie_Tomato_C",
    "category": "Consumable",
    "defaultStackSize": 10
  },
  {
    "label": "Tomato Sauce (Primitive Plus)",
    "classString": "PrimalItemConsumable_TomatoSauce_C",
    "category": "Consumables",
    "defaultStackSize": 100
  },
  {
    "label": "Tomato Seed (Primitive Plus)",
    "classString": "PrimalItemConsumable_Seed_Tomato_C",
    "category": "Seeds",
    "defaultStackSize": 100
  },
  {
    "label": "Touchdown Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Touchdown_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Trading Crate (Bread) (Primitive Plus)",
    "classString": "PrimalItemStructure_TradingCrate_Bread_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Trading Crate (Eggs) (Primitive Plus)",
    "classString": "PrimalItemStructure_TradingCrate_Eggs_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Trading Crate (Fruits Veggies) (Primitive Plus)",
    "classString": "PrimalItemStructure_TradingCrate_FruitsVeggies_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Trading Crate (Primitive Plus)",
    "classString": "PrimalItemStructure_TradingCrate_Small_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Training Dummy",
    "classString": "PrimalItemStructure_TrainingDummy_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Tranq Arrow",
    "classString": "PrimalItemAmmo_ArrowTranq_C",
    "category": "Arrows",
    "defaultStackSize": 100
  },
  {
    "label": "Tranq Spear Bolt",
    "classString": "PrimalItemAmmo_TranqSpearBolt_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Tranq Thalassian Ammo",
    "classString": "PrimalItemAmmo_ThalassianTranqBullet_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Tranquilizer Dart",
    "classString": "PrimalItemAmmo_TranqDart_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Transponder Node",
    "classString": "PrimalItemTransGPSAmmo_C",
    "category": "Tools",
    "defaultStackSize": 100
  },
  {
    "label": "Trap Bait (Primitive Plus)",
    "classString": "PrimalItemResource_CarnivoreBait_C",
    "category": "Resource",
    "defaultStackSize": 100
  },
  {
    "label": "Tree Sap Tap",
    "classString": "PrimalItemStructure_TreeTap_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Trike Egg",
    "classString": "PrimalItemConsumable_Egg_Trike_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Troodon Egg",
    "classString": "PrimalItemConsumable_Egg_Troodon_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Tropeognathus Egg",
    "classString": "PrimalItemConsumable_Egg_Tropeognathus_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Trophy Wall-Mount",
    "classString": "PrimalItemStructure_TrophyWall_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Tudor Bar (L Shaped) (Primitive Plus)",
    "classString": "PrimalItemStructure_TudorBar_L_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Tudor Bar (Primitive Plus)",
    "classString": "PrimalItemStructure_TudorBar_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Turkey Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_turkey_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Turtle Egg",
    "classString": "PrimalItemConsumable_Egg_Turtle_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Tusoteuthis Tentacle",
    "classString": "PrimalItemResource_ApexDrop_Tuso_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Tyrannosaurus Arm",
    "classString": "PrimalItemResource_ApexDrop_Rex_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Umbra Apex Drop",
    "classString": "PrimalItemResource_ApexDrop_Umbra_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Umbra Egg",
    "classString": "PrimalItemConsumable_Egg_Draco_Fertilized_Umbra_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Umbra Scale",
    "classString": "PrimalItemResource_UmbraScale_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Vacuum Compartment",
    "classString": "PrimalItemStructure_UnderwaterBase_C",
    "category": "Tek",
    "defaultStackSize": 5
  },
  {
    "label": "Vacuum Compartment Moonpool",
    "classString": "PrimalItemStructure_UnderwaterBase_Moonpool_C",
    "category": "Tek",
    "defaultStackSize": 5
  },
  {
    "label": "Velonasaur Egg",
    "classString": "PrimalItemConsumable_Egg_Spindles_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Vertical Electrical Cable",
    "classString": "PrimalItemStructure_PowerCableVertical_C",
    "category": "Electricity",
    "defaultStackSize": 100
  },
  {
    "label": "Vessel",
    "classString": "PrimalItemStructure_Vessel_C",
    "category": "Containers",
    "defaultStackSize": 100
  },
  {
    "label": "Viking Beard Facial Hair Style",
    "classString": "PrimalItemConsumable_UnlockHair_Facial_VikingBeard_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "Viking Head Hair Style",
    "classString": "PrimalItemConsumable_UnlockHair_Head_Viking_C",
    "category": "Hairstyles",
    "defaultStackSize": 100
  },
  {
    "label": "VR Boss Flag",
    "classString": "PrimalItemStructure_Flag_VRBoss_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Vulture Egg",
    "classString": "PrimalItemConsumable_Egg_Vulture_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Wall Torch",
    "classString": "PrimalItemStructure_WallTorch_C",
    "category": "Structures",
    "defaultStackSize": 3
  },
  {
    "label": "War Map",
    "classString": "PrimalItemStructure_WarMap_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Wardrums",
    "classString": "PrimalItemStructure_Wardrums_C",
    "category": "Furniture",
    "defaultStackSize": 100
  },
  {
    "label": "Wasteland Lights",
    "classString": "PrimalItemStructure_WastelandLights_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Water Reservoir",
    "classString": "PrimalItemStructure_WaterTank_C",
    "category": "Irrigation",
    "defaultStackSize": 100
  },
  {
    "label": "Water Talon",
    "classString": "PrimalItemResource_ApexDrop_WaterWyvern_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Water Well",
    "classString": "PrimalItemStructure_WaterWell_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Watering Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Watering_C",
    "category": "Emotes",
    "defaultStackSize": 100
  },
  {
    "label": "Weapon Rack (Primitive Plus)",
    "classString": "PrimalItemStructure_StorageShed_Weapon_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Well Bucket (Primitive Plus)",
    "classString": "PrimalItemStructure_WellBucket_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Well Rope (Primitive Plus)",
    "classString": "PrimalItemStructure_ChainPipeStraight_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Wheat Beer (Primitive Plus)",
    "classString": "PrimalItemConsumable_Drink_Beer_C",
    "category": "Consumables",
    "defaultStackSize": 100
  },
  {
    "label": "White Coloring",
    "classString": "PrimalItemDye_White_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Windmill (Primitive Plus)",
    "classString": "PrimalItemStructure_Windmill_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Wishbone",
    "classString": "PrimalItemResource_Wishbone_C",
    "category": "Items",
    "defaultStackSize": 200
  },
  {
    "label": "Wood",
    "classString": "PrimalItemResource_Wood_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Wood Elevator Top Switch",
    "classString": "PrimalItemStructure_WoodElevatorTopSwitch_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Wood Elevator Track",
    "classString": "PrimalItemStructure_WoodElevatorTrack_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Wood Handrail (Primitive Plus)",
    "classString": "PrimalItemStructure_WoodHandrail_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Wood Ocean Platform",
    "classString": "PrimalItemStructure_Wood_OceanPlatform_C",
    "category": "Stone",
    "defaultStackSize": 3
  },
  {
    "label": "Wood Post Fence (Primitive Plus)",
    "classString": "PrimalItemStructure_Fence_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Wood Storage Shed (Primitive Plus)",
    "classString": "PrimalItemStructure_StorageShed_Wood_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Bench",
    "classString": "PrimalItemStructure_Furniture_WoodBench_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Billboard",
    "classString": "PrimalItemStructure_WoodSign_Large_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Cabinet (Primitive Plus)",
    "classString": "PrimalItemStructure_Cabinent_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Catwalk",
    "classString": "PrimalItemStructure_WoodCatwalk_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Ceiling",
    "classString": "PrimalItemStructure_WoodCeiling_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Chair",
    "classString": "PrimalItemStructure_Furniture_WoodChair_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Door",
    "classString": "PrimalItemStructure_WoodDoor_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Doorframe",
    "classString": "PrimalItemStructure_WoodWallWithDoor_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Double Door",
    "classString": "PrimalItemStructure_DoubleDoor_Wood_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Double Doorframe",
    "classString": "PrimalItemStructure_DoubleDoorframe_Wood_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Fence Foundation",
    "classString": "PrimalItemStructure_WoodFenceFoundation_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Fence Support",
    "classString": "PrimalItemStructure_FenceSupport_Wood_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Foundation",
    "classString": "PrimalItemStructure_WoodFloor_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Hatchframe",
    "classString": "PrimalItemStructure_WoodCeilingWithTrapdoor_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Ladder",
    "classString": "PrimalItemStructure_WoodLadder_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Pillar",
    "classString": "PrimalItemStructure_WoodPillar_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Railing",
    "classString": "PrimalItemStructure_WoodRailing_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Ramp",
    "classString": "PrimalItemStructure_WoodRamp_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Sign",
    "classString": "PrimalItemStructure_WoodSign_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Spike Wall",
    "classString": "PrimalItemStructure_WoodSpikeWall_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Staircase",
    "classString": "PrimalItemStructure_WoodStairs_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Stairs",
    "classString": "PrimalItemStructure_Ramp_Wood_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Stairs (One Level) (Primitive Plus)",
    "classString": "PrimalItemStructure_LumberStairs_C",
    "category": "Structure",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Table",
    "classString": "PrimalItemStructure_Furniture_WoodTable_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Trapdoor",
    "classString": "PrimalItemStructure_WoodTrapdoor_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Triangle Ceiling",
    "classString": "PrimalItemStructure_TriCeiling_Wood_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Triangle Foundation",
    "classString": "PrimalItemStructure_TriFoundation_Wood_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Triangle Roof",
    "classString": "PrimalItemStructure_TriRoof_Wood_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Wall",
    "classString": "PrimalItemStructure_WoodWall_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Wall Sign",
    "classString": "PrimalItemStructure_WoodSign_Wall_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Window",
    "classString": "PrimalItemStructure_WoodWindow_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wooden Windowframe",
    "classString": "PrimalItemStructure_WoodWallWithWindow_C",
    "category": "Wood",
    "defaultStackSize": 100
  },
  {
    "label": "Wool",
    "classString": "PrimalItemResource_Wool_C",
    "category": "Resources",
    "defaultStackSize": 200
  },
  {
    "label": "Woolly Rhino Horn",
    "classString": "PrimalItemResource_Horn_C",
    "category": "Resources",
    "defaultStackSize": 20
  },
  {
    "label": "Worm Gum",
    "classString": "PrimalItemConsumable_WormGum_C",
    "category": "Cooking",
    "defaultStackSize": 100
  },
  {
    "label": "Wreath",
    "classString": "PrimalItemStructure_Wreath_C",
    "category": "Structures",
    "defaultStackSize": 100
  },
  {
    "label": "Wyvern Talon",
    "classString": "PrimalItemResource_ApexDrop_FireWyvern_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Yellow Coloring",
    "classString": "PrimalItemDye_Yellow_C",
    "category": "Coloring",
    "defaultStackSize": 100
  },
  {
    "label": "Yurt (Primitive Plus)",
    "classString": "PrimalItemStructure_Yurt_C",
    "category": "Buildings",
    "defaultStackSize": 100
  },
  {
    "label": "Yutyrannus Egg",
    "classString": "PrimalItemConsumable_Egg_Yuty_C",
    "category": "Eggs",
    "defaultStackSize": 100
  },
  {
    "label": "Yutyrannus Lungs",
    "classString": "PrimalItemResource_ApexDrop_Yuty_C",
    "category": "Resources",
    "defaultStackSize": 100
  },
  {
    "label": "Zip-Line Anchor",
    "classString": "PrimalItemAmmo_Zipline_C",
    "category": "Ammunition",
    "defaultStackSize": 100
  },
  {
    "label": "Zombie Emote",
    "classString": "PrimalItemConsumable_UnlockEmote_Zombie_C",
    "category": "Emotes",
    "defaultStackSize": 100
  }
] as const satisfies readonly ItemStackOption[]
