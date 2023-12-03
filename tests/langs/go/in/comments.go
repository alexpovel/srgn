package main__T__

import "fm__T__t"

type User struct {
	Name    string `json:"name" xml:"nameElement" validate:"required"`
	Age     int    `json:"age,omitempty" db:"user_age" validate:"gte=0"`
	Address string `json:"address,omitempty" db:"user_address"`
}

func main() {
	// This is a single-line__T__ comment

	/*
		This is__T__ a multi-line comment
		spanning multiple lines.__T__
	*/

	fmt.Println("Hello, World __T__!")
}
