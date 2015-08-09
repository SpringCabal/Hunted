
function gadget:GetInfo()
  return {
    name      = "Dev Commands",
    desc      = "Adds useful commands.",
    author    = "Google Frog",
    date      = "12 Sep 2011",
    license   = "GNU GPL, v2 or later",
    layer     = 0,
    enabled   = true,  --  loaded by default?
	handler   = true,
  }
end

-------------------------------------------------------------------------------------
-------------------------------------------------------------------------------------

local spIsCheatingEnabled = Spring.IsCheatingEnabled

-------------------------------------------------------------------------------------
-------------------------------------------------------------------------------------

function GG.TableEcho(data, indent)
	indent = indent or ""
	for name, v in pairs(data) do
		local ty =  type(v)
		if ty == "table" then
			Spring.Echo(indent .. name .. " = {")
			GG.TableEcho(v, indent .. "    ")
			--Spring.Echo("Spring.Echo(indent .. "}"" .. )
		elseif ty == "boolean" then
			Spring.Echo(indent .. name .. " = " .. (v and "true" or "false"))
		else
			Spring.Echo(indent .. name .. " = " .. v)
		end
	end
end

function GG.UnitEcho(unitID, st)
	st = st or unitID
	if Spring.ValidUnitID(unitID) then
		local x,y,z = Spring.GetUnitPosition(unitID)
		Spring.MarkerAddPoint(x,y,z, st)
	else
		Spring.Echo("Invalid unitID")
		Spring.Echo(unitID)
		Spring.Echo(st)
	end
end

-------------------------------------------------------------------------------------
-------------------------------------------------------------------------------------

if (gadgetHandler:IsSyncedCode()) then
   

-- '/luarules circle'
-- '/luarules give'
-- '/luarules gk'
-- '/luarules clear'
-- '/luarules restart'

-------------------------------------------------------------------------------------
-------------------------------------------------------------------------------------

-- UnitName TeamID Number Radius [Xpos Zpos]
-- For example '/luarules circle corllt 1 60 420 3200 3200'
local function circleGive(cmd, line, words, player)
	if not (spIsCheatingEnabled() and #words >= 4) then 
		return
	end
	local unitName = words[1]
	local team = math.abs(tonumber(words[2]) or 0)
	local count = math.floor(tonumber(words[3]) or 0)
	local radius = math.abs(tonumber(words[4]) or 0)
	local ox = tonumber(words[5]) or Game.mapSizeX/2
	local oz = tonumber(words[6]) or Game.mapSizeZ/2
	if not (type(unitName) == "string" and UnitDefNames[unitName] and team >= 0 and count > 0 and radius > 0) then
		return
	end
	local unitDefID = UnitDefNames[unitName].id
	local increment = 2*math.pi/count
	for i = 1, count do
		local angle = i*increment
		local x = ox + math.cos(angle)*radius
		local z = oz + math.sin(angle)*radius
		local y = Spring.GetGroundHeight(x,z)
		Spring.CreateUnit(unitDefID, x, y, z, 0, team, false)
	end
end

local function give(cmd,line,words,player)
	if spIsCheatingEnabled() then
		local buildlist = UnitDefNames["armcom1"].buildOptions
		local INCREMENT = 128
		for i = 1, #buildlist do
			local udid = buildlist[i]
			local x, z = INCREMENT, i*INCREMENT
			local y = Spring.GetGroundHeight(x,z)
			Spring.CreateUnit(udid, x, y, z, 0, 0, false)
			local ud = UnitDefs[udid]
			if ud.buildOptions and #ud.buildOptions > 0 then
				local sublist = ud.buildOptions
				for j = 1, #sublist do
					local subUdid = sublist[j]
					local x2, z2 = (j+1)*INCREMENT, i*INCREMENT
					local y2 = Spring.GetGroundHeight(x2,z2)
					Spring.CreateUnit(subUdid, x2, y2, z2+32, 0, 0, false)
					--Spring.CreateUnit(subUdid, x2+32, y2, z2, 1, 0, false)
					--Spring.CreateUnit(subUdid, x2, y2, z2-32, 2, 0, false)
					--Spring.CreateUnit(subUdid, x2-32, y2, z2, 3, 0, false)
				end	
			end
		end
	end
end

local function gentleKill(cmd,line,words,player)
	if spIsCheatingEnabled() then
		local units = Spring.GetAllUnits()
		for i=1, #units do
			local unitID = units[i]
			Spring.SetUnitHealth(unitID,0.1)
			Spring.AddUnitDamage(unitID,1, 0, nil, -7)
		end
	end
end

local function clear(cmd,line,words,player)
	if spIsCheatingEnabled() then
		local units = Spring.GetAllUnits()
		for i=1, #units do
			local unitID = units[i]
			Spring.DestroyUnit(unitID, false, true)
		end
		local features = Spring.GetAllFeatures()
		for i=1, #features do
			local featureID = features[i]
			Spring.DestroyFeature(featureID)
		end
	end
end

local function restart(cmd,line,words,player)
	if spIsCheatingEnabled() then
		local units = Spring.GetAllUnits()
		
		local teams = Spring.GetTeamList()
		for i=1,#teams do
			local teamID = teams[i]
			if GG.startUnits[teamID] and GG.CommanderSpawnLocation[teamID] then
				local spawn = GG.CommanderSpawnLocation[teamID]
				local unitID = GG.DropUnit(GG.startUnits[teamID], spawn.x, spawn.y, spawn.z, spawn.facing, teamID, nil, 0)
				Spring.SetUnitRulesParam(unitID, "facplop", 1, {inlos = true})
			end
		end
		
		for i=1, #units do
			local unitID = units[i]
			Spring.DestroyUnit(unitID, false, true)
		end
		local features = Spring.GetAllFeatures()
		for i=1, #features do
			local featureID = features[i]
			Spring.DestroyFeature(featureID)
		end
	end
end

local function bisect(cmd,line,words,player)
	local increment = math.abs(tonumber(words[1]) or 1)
	local offset = math.floor(tonumber(words[2]) or 0)
	local invert = (math.abs(tonumber(words[3]) or 0) == 1) or false
	
	--[[
	local occured = {}
	for i = 1, #syncedGadgetList do
		occured[syncedGadgetList[i] ] = true
	end
	for i = 1, #unsyncedGadgetList do 
		if not occured[unsyncedGadgetList[i] ] then
			syncedGadgetList[#syncedGadgetList+1] = unsyncedGadgetList[i]
		end
	end
	
	for i = 1, #syncedGadgetList do
		Spring.Echo("\"" .. syncedGadgetList[i] .. "\",")
	end
	--]]
	
	for i = 1, #gadgetList do
		if i >= offset and (offset-i)%increment == 0 then
			if not invert then
				gadgetHandler:GotChatMsg("disablegadget " .. gadgetList[i], 0)
			end
		elseif invert then
			gadgetHandler:GotChatMsg("disablegadget " .. gadgetList[i], 0)
		end
	end
end

function gadget:Initialize()
	gadgetHandler.actionHandler.AddChatAction(self,"bisect",bisect,"Bisect gadget disables.")
	gadgetHandler.actionHandler.AddChatAction(self,"circle",circleGive,"Gives a bunch of units in a circle.")
	gadgetHandler.actionHandler.AddChatAction(self,"give",give,"Like give all but without all the crap.")
	gadgetHandler.actionHandler.AddChatAction(self,"gk",gentleKill,"Gently kills everything.")
	gadgetHandler.actionHandler.AddChatAction(self,"clear",clear,"Clears all units and wreckage.")
	gadgetHandler.actionHandler.AddChatAction(self,"restart",restart,"Gives some commanders and clears everything else.")
end

-------------------------------------------------------------------------------------
-------------------------------------------------------------------------------------
else -- UNSYNCED
-------------------------------------------------------------------------------------
-------------------------------------------------------------------------------------

local function bisect(cmd,line,words,player)
	local increment = math.abs(tonumber(words[1]) or 1)
	local offset = math.floor(tonumber(words[2]) or 0)
	local invert = (math.abs(tonumber(words[3]) or 0) == 1) or false
	
	for i = 1, #gadgetList do
		if i >= offset and (offset-i)%increment == 0 then
			if not invert then
				gadgetHandler:GotChatMsg("disablegadget " .. gadgetList[i], 0)
			end
		elseif invert then
			gadgetHandler:GotChatMsg("disablegadget " .. gadgetList[i], 0)
		end
	end
	collectgarbage("collect")
end

local function gc()
	collectgarbage("collect")
end

function gadget:Initialize()
	gadgetHandler.actionHandler.AddChatAction(self,"bisect",bisect,"Bisect gadget disables.")
	gadgetHandler.actionHandler.AddChatAction(self,"gc",gc,"Garbage collect.")
end

end
   