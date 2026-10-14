-- Weapon defs built with Weapon:New{} inherit the Def helper methods (New,
-- Clone, Append) as plain table entries, which the engine then reports as
-- unknown tags. Strip every function value before the defs reach the engine.
local function StripFunctions(t, seen)
	if seen[t] then
		return
	end
	seen[t] = true
	for key, value in pairs(t) do
		if type(value) == "function" then
			t[key] = nil
		elseif type(value) == "table" then
			StripFunctions(value, seen)
		end
	end
end

for _, wd in pairs(WeaponDefs) do
	StripFunctions(wd, {})
end
