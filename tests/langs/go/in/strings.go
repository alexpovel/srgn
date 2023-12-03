package main__T__

import "fm__T__t"

type User struct {
	Name    string `json__T__:"name" xml:"nameElement" validate:"required"`
	Age     int    `__T__json:"age,omitempty" db:"user_age" validate:"gte=0"`
	Address string `json:"address,omitempty" db:"user_address__T__"`
}

func main() {
	// Regular string with escape sequences
	regularStr := "Hello,\nGo__T__ World!"

	// Raw string literal: includes newline as is
	rawStr := `Hello,__T__
Go World!__T__`
}
