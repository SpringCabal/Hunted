if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Shotgun",
		desc	= "Placeholder for shotgun shooting gadget.",
		author	= "Google Frog",
		date	= "8 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 20,
		enabled = true
	}
end

local shotgunDefId = UnitDefNames["shotgun"].id
local shotgunID = nil
local targetx, targety, targetz
local COB_ANGULAR = 182
-------------------------------------------------------------------
-------------------------------------------------------------------

local function explode(div,str)
	if (div=='') then return 
		false 
	end
	local pos,arr = 0,{}
	-- for each divider found
	for st,sp in function() return string.find(str,div,pos,true) end do
		table.insert(arr,string.sub(str,pos,st-1)) -- Attach chars left of current divider
		pos = sp + 1 -- Jump past current divider
	end
	table.insert(arr,string.sub(str,pos)) -- Attach chars right of last divider
	return arr
end

-------------------------------------------------------------------
-- Configuration
-------------------------------------------------------------------

local shotgunAttributes = {
	radius = 300,
	radiusSq = 300^2,
	edgeMagnitude = 0,
	proximityMagnitude = 150,
}

local torchAttributes = {
	radius = 200,
	radiusSq = 200^2,
	edgeMagnitude = 0.1,
	proximityMagnitude = 1.8,
}

-------------------------------------------------------------------
-- Spawning Projectiles
-------------------------------------------------------------------

local function Norm(x, y, z)
	local size = math.sqrt(x * x + y * y + z * z)
	return x / size, y / size, z / size
end

local function SpawnShot(def, spawnx, spawny, spawnz, dx, dy, dz)
	local ex, ey, ez = (math.random() * 2 - 1) * def.sprayAngle, (math.random() * 2 - 1) * def.sprayAngle, (math.random() * 2 - 1) * def.sprayAngle
	local dirx, diry, dirz = Norm(dx + ex, dy + ey, dz + ez)
	local v = def.projectilespeed
	
	local params = {
		pos = {spawnx, spawny, spawnz},
		speed = {dirx * v, diry * v, dirz * v},
		owner  = shotgunID,
	}
	Spring.SpawnProjectile(def.id, params)
end

local function FireShotgun(x, y, z)
	if not shotgunID then
		return
	end
	
	local ammo = Spring.GetGameRulesParam("shotgun_ammo") or 0
	if ammo < 1 then
		return
	end
	Spring.SetGameRulesParam("shotgun_ammo", ammo - 1)
	
	GG.ScareRabbitsInArea(x, z, shotgunAttributes)
	
	local shotgunDef = WeaponDefNames.shotgun
	local flare = Spring.GetUnitPieceMap(shotgunID).flare
	local spawnx, spawny, spawnz = Spring.GetUnitPiecePosDir(shotgunID, flare)
	local dx, dy, dz = Norm(x - spawnx, y - spawny, z - spawnz)
	
	for i = 1, shotgunDef.projectiles do
		SpawnShot(shotgunDef, spawnx, spawny, spawnz, dx, dy, dz)
	end
	
	Spring.SetUnitVelocity(shotgunID, -dx * 20, -dy * 20, -dz * 20)
	local env = Spring.UnitScript.GetScriptEnv(shotgunID)
	Spring.UnitScript.CallAsUnit(shotgunID, env.Fire)
	Spring.PlaySoundFile("sounds/shotgun1.wav", 20, spawnx, spawny, spawnz)
	Spring.GiveOrderToUnit(shotgunID, CMD.STOP, {}, {})
	if targetx then
		Spring.GiveOrderToUnit(shotgunID, CMD.MOVE, {targetx + 100, targety, targetz - 100}, {})
	end
end

-------------------------------------------------------------------
-- Handling unit
-------------------------------------------------------------------

local function MoveShotgun(x, y, z)
	targetx, targety, targetz = x, y, z
	if not shotgunID then
		if gameStarted then
			Spring.CreateUnit(shotgunDefId, x + 50, y + 100, z + 50, 0, Spring.GetGaiaTeamID())
		end
		return
	end
	Spring.GiveOrderToUnit(shotgunID, CMD.MOVE, {targetx + 100, targety, targetz - 100}, {})
	
	torchScaryArea.x = x
	torchScaryArea.z = z
end

function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == shotgunDefId then
		shotgunID = unitID
		Spring.GiveOrderToUnit(unitID, CMD.IDLEMODE, {0}, {}) --no land
	end
end

function gadget:GameStart()
	gameStarted = true
end

function gadget:GameFrame(n)
	if not shotgunID or not targetx then
		return
	end
	
	local ux, uy, uz = Spring.GetUnitPosition(shotgunID)
	local dx, dy, dz = targetx - ux, uy - targety, targetz - uz
	local newHeading = math.deg(math.atan2(dx, dz)) * COB_ANGULAR
	local dist = math.sqrt(dx * dx + dz * dz)
	local pitch = math.atan2(dy, dist)
	
	Spring.SetUnitCOBValue(shotgunID, COB.HEADING, newHeading)
	local env = Spring.UnitScript.GetScriptEnv(shotgunID)
	Spring.UnitScript.CallAsUnit(shotgunID, env.SetPitch, pitch)
end

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
	
	torchScaryArea = GG.AddScaryArea({x = 0, z = 0, attributes = torchAttributes})
end

-------------------------------------------------------------------
-- Handling messages
-------------------------------------------------------------------

function HandleLuaMessage(msg)
	local msg_table = explode('|', msg)
	if msg_table[1] == 'shotgun' then
		local x = tonumber(msg_table[2])
		local y = tonumber(msg_table[3])
		local z = tonumber(msg_table[4])
		
		FireShotgun(x, y, z)
	end
	if msg_table[1] == 'movegun' then
		local x = tonumber(msg_table[2])
		local y = tonumber(msg_table[3])
		local z = tonumber(msg_table[4])
		
		MoveShotgun(x, y, z)
	end
end

function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end

