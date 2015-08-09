 include "lib_OS.lua"
 include "lib_UnitScript.lua"
 include "lib_Build.lua" 
--pieces
Head=piece"Head"
WiggleTail=piece"WiggleTail"
center=piece"center"
piecesTable=makeKeyPiecesTable(unitID, piece)

--Signals
local SIG_MOVE=2

function script.HitByWeapon (x, z, weaponDefID, damage) 
end


function script.Create()
	StartThread(MoveAnimationController)
end

function script.Killed(recentDamage, _)
	return 0
end

TurnTableUpMoveUp=
{
	[piecesTable["Body"] ]  = {axis = x_axis, deg = -14}, 
	[piecesTable["BLeg1"] ] = {axis = x_axis, deg = 47}, 
	[piecesTable["BLeg2"] ] = {axis = x_axis, deg = 51}, 
	[piecesTable["FLeg1"] ] = {axis = x_axis, deg = 44}, 
	[piecesTable["FLeg1"] ] = {axis = y_axis, deg = -21},  
	[piecesTable["FLeg2"] ] = {axis = x_axis, deg = 44}, 
	[piecesTable["FLeg2"] ] = {axis = y_axis, deg = 24}, 
	[piecesTable["Head"] ]  = {axis = x_axis, deg = -27}
}                                                  

TurnTableUpMoveDown=
{
	[piecesTable["Body"]  ] = {axis = x_axis, deg= -14}, 
	[piecesTable["BLeg1"] ] = {axis = x_axis, deg= 9}, 
	[piecesTable["BLeg2"] ] = {axis = x_axis, deg= 10}, 
	[piecesTable["FLeg1"] ] = {axis = x_axis, deg= 44},  
	[piecesTable["FLeg2"] ] = {axis = x_axis, deg= -32}, 
	[piecesTable["Head"]  ] = {axis = x_axis, deg= 17}
}

function MoveAnimation()
	Move(center, y_axis, 5, 2.5)
	turnSyncInTimeTable(TurnTableUpMoveUp, 700)
	Sleep(650)
	WaitForMove(center, y_axis)

	Move(center, y_axis, 0, 2.5)
	turnSyncInTimeTable(TurnTableUpMoveDown, 420)
	Sleep(400)
	WaitForMove(center, y_axis)
end

function MoveAnimationController()

	while true do
		if boolMoving == true then 
			MoveAnimation()
		end
			if boolMoving == false then
				reseT(piecesTable, 7)
			if maRa() == true then 
				idle()
			end
		end
		Sleep(100)
	end
end

function idle()
	val = math.random(1, 6)
	for i = 1, val do
		deg = math.random(10, 35)
		Turn(Head, y_axis, math.rad(deg), 9)
		Turn(WiggleTail, x_axis, math.rad(math.random(-10, 10), 8))
		WaitForTurn(Head, y_axis)
		Turn(Head, y_axis, math.rad(deg*-1), 9)

		Turn(WiggleTail, x_axis, math.rad(math.random(-10, 10), 8))
		WaitForTurn(Head, y_axis)
	end
	reseT(piecesTable)
end

function script.StartMoving()
	boolMoving = true
	Signal(SIG_MOVE)
end

function AttachCarrot(carrotID)
	Spring.UnitScript.AttachUnit(center, carrotID)
end

function MoveEnded()
	Signal(SIG_MOVE)
	SetSignalMask(SIG_MOVE)
	Sleep(500)
	boolMoving = false
end

function script.StopMoving()
	StartThread(MoveEnded)
end
