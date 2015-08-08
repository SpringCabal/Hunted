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

function HandleLuaMessage(msg)
	local msg_table = explode('|', msg)
	if msg_table[1] ~= 'shotgun' then
		return
	end

	local x = tonumber(msg_table[2])
	local y = tonumber(msg_table[3])
	local z = tonumber(msg_table[4])
	
	Spring.MarkerAddPoint(x,y,z, "Boom")
	GG.ScareRabbitsInArea(x, z, shotgunAttributes)
end


function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end