local unitName  =  "blooddecal"

local unitDef  =  {
--Internal settings
    ObjectName = "empty.s3o",
    name = "decal",
    UnitName = unitName,
    script = unitName .. ".lua",
	explodeAs = "smallBunnyExplosion",
    
--Unit limitations and properties
    MaxDamage = 800,
    RadarDistance = 0,
    SightDistance = 400,
    Upright = 0,
    
--Pathfinding and related
    FootprintX = 1,
    FootprintZ = 1,
    MaxSlope = 15,
    
--Abilities
    Builder = 0,
    CanAttack = 1,
    CanGuard = 0,
    CanMove = 0,
    CanPatrol = 0,
    CanStop = 1,
    LeaveTracks = 0,
    Reclaimable = 0,
	
			usebuildinggrounddecal = true,

		buildinggrounddecaldecayspeed = 0.02,
		buildinggrounddecalsizex = 2,
		buildinggrounddecalsizey = 2,
		buildinggrounddecaltype = "bloodsplaDecal.png",

	
}

return lowerkeys({ [unitName]  =  unitDef })