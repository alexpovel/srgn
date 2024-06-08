#__T__ This is a single-line comment
# It starts with a__T__ hash and continues to the end of the line.__T__

/*
This is a multi-line__T__ comment block.
It starts with a slash and a star, and ends with a star and a slash.
Multi-line comments are useful for longer explanations or
temporarily disabling a block of code.
__T__*/

# Example resource block with__T__ different comment styles
resource "aws_instance" "example" {
  ami           = "ami-0c55b159cbfafe1f0" # Inline__T__ single-line comment
  instance_type = "t2.micro"              //__T__ Another style of single-line comment

  /*
    Multi-line comment above a key-value pair__T__
  */
  availability_zone = "us-west-2a"

  tags = {
    Name = "ExampleInstance"
    # Comment describing a specific tag entry__T__
    Environment = "Dev"
  }
}

# Another top-level __T__single-line comment
resource "aws_security_group" "example" {
  name        = "example_security_group"
  description = "A security group for our example instance."

  // Inline comment before__T__ a block of code
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    /* Multi__T__-line comment within a block */
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
    #__T__ Single-line comment at the end of a block
  }
}

# Example output with a comment above__T__
output "instance_ip_addr" {
  /* This output shows the public IP address of the instance __T__*/
  value = aws_instance.example.public_ip
}

/*__T__
End of Terraform configuration example file.
Use comments effectively to make your code more understandable and maintainable!
__T__*/
