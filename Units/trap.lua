local unitName  =  "trap"

local unitDef  =  {
--Internal settings
    ObjectName = "Trap2.s3o",
    name = unitName,
    UnitName = unitName,
    script = unitName .. ".lua",
    
--Unit limitations and properties
    MaxDamage = 800,
    RadarDistance = 0,
    SightDistance = 400,
    Upright = 0,
    
--Pathfinding and related
    FootprintX = 1,
    FootprintZ = 1,
    MaxSlope = 90,
	
	
    
--Abilities
    Builder = 0,
    CanAttack = 1,
    CanGuard = 0,
    CanMove = 0,
    CanPatrol = 0,
    CanStop = 1,
    LeaveTracks = 0,
    Reclaimable = 0,
	
--Hitbox
--    collisionVolumeOffsets    =  "0 0 0",
--    collisionVolumeScales     =  "20 20 20",
--    collisionVolumeTest       =  1,
--    collisionVolumeType       =  "box",
}

return lowerkeys({ [unitName]  =  unitDef })