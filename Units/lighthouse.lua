local unitName  =  "lighthouse"

local unitDef  =  {
--Internal settings
    ObjectName = "Tower.s3o",
    name = "Lighthouse",
    UnitName = unitName,
    script = unitName .. ".lua",
    
--Unit limitations and properties
    MaxDamage = 50000*30, -- 30 shotgun pellets
    RadarDistance = 0,
    SightDistance = 400,
    Upright = 0,
    
--Pathfinding and related
    FootprintX = 3,
    FootprintZ = 3,
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
	
--Hitbox
--    collisionVolumeOffsets    =  "0 0 0",
--    collisionVolumeScales     =  "20 20 20",
--    collisionVolumeTest       =  1,
--    collisionVolumeType       =  "box",
}

return lowerkeys({ [unitName]  =  unitDef })