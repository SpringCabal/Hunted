local shell = piece "Shell"
Spring.Echo(shell)
function script.Create()
	Hide(shell)
end

function Fire()
	Explode(shell, 0)
end
