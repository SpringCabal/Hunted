local unitName  =  "blooddecal"

local unitDef  =  {
--Internal settings
    ObjectName = "empty.s3o",
    name = "decal",
    UnitName = unitName,
    script = unitName .. ".lua",
	explodeAs = "smallBunnyExplosion",
    
--Unit limitations and properties
    MaxDamage = 1,
    RadarDistance = 0,
    SightDistance = 400,
    Upright = 0,
	
-- Transporting
	releaseHeld = true,
	holdSteady = true,
	transportCapacity = 10,
	transportSize = 10,
	
--Pathfinding and related
    Acceleration = 0.2,
    BrakeRate = 0.1,
    FootprintX = 2,
    FootprintZ = 2,
    MaxSlope = 15,
    MaxVelocity = 32, -- max velocity is so high because Spring cannot increase max velocity beyond its maximum.
    MaxWaterDepth = 20,
    MovementClass = "Bot2x2",
	TurnInPlace = false,
	TurnInPlaceSpeedLimit = 1.6, 
	turnInPlaceAngleLimit = 90,
    TurnRate = 3000,
	
	customParams = {
		turnaccel = 500
	},
    
--Abilities
    Builder = 0,
    CanAttack = 0,
    CanGuard = 0,
    CanMove = 0,
    CanPatrol = 0,
    CanStop = 0,
    LeaveTracks = 0,
    Reclaimable = 0,
	
--Hitbox

	
			usebuildinggrounddecal = true,

		buildinggrounddecaldecayspeed = 0.02,
		buildinggrounddecalsizex = 2,
		buildinggrounddecalsizey = 2,
		buildinggrounddecaltype = "bloodsplaDecal.png",

	
}

return lowerkeys({ [unitName]  =  unitDef })