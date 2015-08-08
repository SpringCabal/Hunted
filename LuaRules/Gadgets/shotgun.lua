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
	}
	Spring.SpawnProjectile(def.id, params)
end

local function SpawnShotgun(x, y, z)
	local shotgunDef = WeaponDefNames.shotgun
	local spawnx, spawny, spawnz = x + 30, y + 100, z - 30
	local dx, dy, dz = Norm(x - spawnx, y - spawny, z - spawnz)
	
	for i = 1, shotgunDef.projectiles do
		SpawnShot(shotgunDef, spawnx, spawny, spawnz, dx, dy, dz)
	end
end

function HandleLuaMessage(msg)
	local msg_table = explode('|', msg)
	if msg_table[1] ~= 'shotgun' then
		return
	end

	local x = tonumber(msg_table[2])
	local y = tonumber(msg_table[3])
	local z = tonumber(msg_table[4])
	
	SpawnShotgun(x, y, z)
	GG.ScareRabbitsInArea(x, z, shotgunAttributes)
end


function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end