package main

type User struct {
	Name     string `json__T__:"name" xml:"nameElement" validate:"required"`
	Password string `json:"-" xml:"__T__"`
}
