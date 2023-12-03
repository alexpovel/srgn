package main

type User struct {
	Name     string `json:"name" xml:"nameElement" validate:"required"`
	Password string `json:"-" xml:""`
}
