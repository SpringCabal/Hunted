--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
   return {
      name      = "Structure Placer",
      desc      = "Places structures specified by the player.",
      author    = "Google Frog",
      date      = "9 August 2015",
      license   = "GNU GPL, v2 or later",
      layer     = 20,
      enabled   = true
   }
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

--SYNCED
if (not gadgetHandler:IsSyncedCode()) then
   return false
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

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
-- Handling messages
-------------------------------------------------------------------

function HandleLuaMessage(msg)
	local msg_table = explode('|', msg)
	if msg_table[1] == 'placeStructure' then
		local unitDefID = tonumber(msg_table[2])
		local x = tonumber(msg_table[3])
		local y = tonumber(msg_table[4])
		local z = tonumber(msg_table[5])
		
		local unitID = Spring.CreateUnit(unitDefID, x, y, z, 0, 0, false, false)
		Spring.SetUnitRotation(unitID, 0, math.random()*2*math.pi, 0)
	end
end

function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end
