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
		layer	= 0,
		enabled = true
	}
end

local shotgunDefId = UnitDefNames["shotgun"].id
local shotgunID = nil
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
-- Spawning Projectiles
-------------------------------------------------------------------

local shotgunAttributes = {
	radius = 300,
	radiusSq = 300^2,
	edgeMagnitude = 50,
	proximityMagnitude = 300,
}

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
	
	local shotgunDef = WeaponDefNames.shotgun
	local flare = Spring.GetUnitPieceMap(shotgunID).flare
	local spawnx, spawny, spawnz = Spring.GetUnitPiecePosDir(shotgunID, flare)
	local dx, dy, dz = Norm(x - spawnx, y - spawny, z - spawnz)
	
	for i = 1, shotgunDef.projectiles do
		SpawnShot(shotgunDef, spawnx, spawny, spawnz, dx, dy, dz)
	end
	
	Spring.SetUnitVelocity(shotgunID, -dx * 10, -dy * 10, -dz * 10)
	local env = Spring.UnitScript.GetScriptEnv(shotgunID)
	Spring.UnitScript.CallAsUnit(shotgunID, env.Fire)
end

-------------------------------------------------------------------
-- Handling unit
-------------------------------------------------------------------
local targetx, targety, targetz
local COB_ANGULAR = 182
local function MoveShotgun(x, y, z)
	targetx, targety, targetz = x, y, z
	if not shotgunID then
		if gameStarted then
			Spring.CreateUnit(shotgunDefId, x + 50, y + 100, z + 50, 0, Spring.GetGaiaTeamID())
		end
		return
	end
	Spring.GiveOrderToUnit(shotgunID, CMD.MOVE, {targetx + 50, targety + 100, targetz + 50}, {})
end

function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == shotgunDefId then
		shotgunID = unitID
		Spring.MoveCtrl.Disable(shotgunID)
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
	local dx, dz = targetx - ux, targetz - uz
	local newHeading = math.deg(math.atan2(dx, dz)) * COB_ANGULAR
	
	Spring.SetUnitCOBValue(shotgunID, COB.HEADING, newHeading)
end

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
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
		GG.ScareRabbitsInArea(x, z, shotgunAttributes)
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

