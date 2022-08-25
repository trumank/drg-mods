// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Framework/Commands/Commands.h"
#include "BPGenStyle.h"

class FBPGenCommands : public TCommands<FBPGenCommands>
{
public:

	FBPGenCommands()
		: TCommands<FBPGenCommands>(TEXT("BPGen"), NSLOCTEXT("Contexts", "BPGen", "BPGen Plugin"), NAME_None, FBPGenStyle::GetStyleSetName())
	{
	}

	// TCommands<> interface
	virtual void RegisterCommands() override;

public:
	TSharedPtr< FUICommandInfo > OpenPluginWindow;
};