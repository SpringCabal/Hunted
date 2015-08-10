if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Ammo Handler",
		desc	= "Maintains ammo stockpiles and provides income.",
		author	= "Google Frog",
		date	= "10 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 0,
		enabled = true
	}
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local maxCarrots = 100
local reloadTimes = {
	shotgun_ammo = 6,
	mine_ammo = 1/2,
}

local maxAmmos = {
	shotgun_ammo = 100,
	mine_ammo = 100,
}

local maxShotgunReloadTime = 5 -- frames
local maxMineReloadTIme = 150 -- frames

local function UpdateAmmoStockpile(ammoType, mult)
	local count = Spring.GetGameRulesParam(ammoType)
	local newAmmo = math.min(maxAmmos[ammoType], count + reloadTimes[ammoType]*mult)
	Spring.SetGameRulesParam(ammoType, newAmmo)
end

local function UpdateAmmo()
	local carrotCount = Spring.GetGameRulesParam("carrot_count")
	local reloadMult = carrotCount/maxCarrots
	
	UpdateAmmoStockpile("shotgun_ammo", reloadMult)
	UpdateAmmoStockpile("mine_ammo", reloadMult)
end

local function AddAmmo(ammoType, val)
	local count = Spring.GetGameRulesParam(ammoType)
	local newAmmo = math.min(maxAmmos[ammoType], count + val)
	Spring.SetGameRulesParam(ammoType, newAmmo)
end

function gadget:Initialize()
	GG.UpdateAmmo = UpdateAmmo
	GG.AddAmmo = AddAmmo

	Spring.SetGameRulesParam("shotgun_ammo", 100)
	Spring.SetGameRulesParam("mine_ammo", 5)
end